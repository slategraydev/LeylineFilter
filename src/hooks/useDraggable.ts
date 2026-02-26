// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import { useState, useCallback, useEffect, useRef } from "react";
import { GRID_UNIT_PX } from "../constants";
import { GridPosition } from "../types";

/**
 * # useDraggable Hook
 * Provides smooth dragging with scale-invariant mouse tracking.
 * Uses an absolute anchor approach to keep the grab point locked under the cursor.
 */
export function useDraggable(
  initialGridPosition: GridPosition,
  onDragEnd?: (pos: GridPosition) => void,
  onDrag?: (
    pos: GridPosition,
    rawOffset?: { x: number; y: number },
    continuousPos?: GridPosition,
  ) => void,
  scale: number = 1.0,
  onDragStart?: () => void,
) {
  const [dragOffset, setDragOffset] = useState<{ x: number; y: number }>({
    x: 0,
    y: 0,
  });
  const [isDragging, setIsDragging] = useState(false);

  // We store the grab point in LOCAL units relative to the module's top-left.
  // This is scale-invariant.
  const grabPointLocal = useRef<{ x: number; y: number }>({ x: 0, y: 0 });

  // Use a ref for scale to avoid closure staleness during dragging
  const scaleRef = useRef(scale);
  useEffect(() => {
    scaleRef.current = scale;
  }, [scale]);

  // Keep track of the initial grid position at the start of a drag
  const startGridPos = useRef<GridPosition>(initialGridPosition);
  const currentGridPos = useRef<GridPosition>(initialGridPosition);

  useEffect(() => {
    if (!isDragging) {
      setDragOffset({ x: 0, y: 0 });
      currentGridPos.current = initialGridPosition;
    }
  }, [initialGridPosition, isDragging]);

  const onPointerDown = useCallback(
    (e: React.PointerEvent) => {
      if (e.button !== 0) return;

      const target = e.target as HTMLElement;
      if (
        target.tagName === "BUTTON" ||
        target.tagName === "INPUT" ||
        target.tagName === "SELECT" ||
        target.closest(".switch") ||
        target.closest(".slider") ||
        target.closest(".custom-select")
      ) {
        return;
      }

      // Find the grid origin (grid-inner) to calculate local coordinates
      const gridInner = target.closest(".grid-inner");
      if (!gridInner) return;

      setIsDragging(true);
      startGridPos.current = initialGridPosition;

      // Capture the pointer to keep receiving events even if the mouse leaves the window
      if ((e.currentTarget as HTMLElement).setPointerCapture) {
        (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
      }

      const rect = gridInner.getBoundingClientRect();
      const gridUnit =
        parseFloat(
          (gridInner as HTMLElement).style.getPropertyValue("--grid-unit"),
        ) || GRID_UNIT_PX;

      // Calculate where on the module we clicked in logical grid units
      const mouseXLocal = (e.clientX - rect.left) / gridUnit;
      const mouseYLocal = (e.clientY - rect.top) / gridUnit;

      // grabOffset_units = mouseX_local_units - moduleLeft_units
      grabPointLocal.current = {
        x: mouseXLocal - initialGridPosition.gx,
        y: mouseYLocal - initialGridPosition.gy,
      };
      if (onDragStart) onDragStart();
      e.preventDefault();
    },
    [initialGridPosition],
  );

  useEffect(() => {
    if (!isDragging) return;

    const handlePointerMove = (e: PointerEvent) => {
      const moduleElement =
        document.querySelector(`[data-dragging="true"]`) ||
        (e.target as HTMLElement).closest(".module-card");
      const gridInner = moduleElement?.closest(".grid-inner") as HTMLElement;
      if (!gridInner) return;

      const rect = gridInner.getBoundingClientRect();
      const gridUnit =
        parseFloat(gridInner.style.getPropertyValue("--grid-unit")) ||
        GRID_UNIT_PX;

      // Calculate where the module top-left should be in PIXELS relative to the grid origin
      // so that the grab point (in pixels) stays under the current mouse position.
      const targetXPixel =
        e.clientX - rect.left - grabPointLocal.current.x * gridUnit;
      const targetYPixel =
        e.clientY - rect.top - grabPointLocal.current.y * gridUnit;

      const startXPixel = startGridPos.current.gx * gridUnit;
      const startYPixel = startGridPos.current.gy * gridUnit;

      // The offset is the difference from the STARTING position in "logical" pixels (where 1 logical pixel = 1/gridUnit units)
      // but actually it's easier to just report the logical units directly to App.
      const nextOffset = {
        x: ((targetXPixel - startXPixel) / gridUnit) * GRID_UNIT_PX,
        y: ((targetYPixel - startYPixel) / gridUnit) * GRID_UNIT_PX,
      };

      setDragOffset(nextOffset);

      // Logical grid positions
      const continuousGx = targetXPixel / gridUnit;
      const continuousGy = targetYPixel / gridUnit;

      const snappedGx = Math.max(0, Math.round(continuousGx));
      const snappedGy = Math.max(0, Math.round(continuousGy));

      const newPos = { gx: snappedGx, gy: snappedGy };
      const continuousPos = { gx: continuousGx, gy: continuousGy };

      if (onDrag) onDrag(newPos, nextOffset, continuousPos);

      if (
        newPos.gx !== currentGridPos.current.gx ||
        newPos.gy !== currentGridPos.current.gy
      ) {
        currentGridPos.current = newPos;
      }
    };

    const handlePointerUp = () => {
      setIsDragging(false);
      if (onDragEnd) {
        onDragEnd(currentGridPos.current);
      }
    };

    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", handlePointerUp);
    window.addEventListener("pointercancel", handlePointerUp);

    return () => {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
      window.removeEventListener("pointercancel", handlePointerUp);
    };
  }, [isDragging, onDrag, onDragEnd]);

  return {
    dragOffset,
    isDragging,
    onPointerDown,
  };
}

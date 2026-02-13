// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import { useState, useCallback, useEffect, useRef } from 'react';
import { GRID_UNIT_PX } from '../constants';

export interface GridPosition {
  gx: number;
  gy: number;
}

/**
 * # useDraggable Hook
 * Provides smooth dragging with scale-invariant mouse tracking.
 * Uses an absolute anchor approach to keep the grab point locked under the cursor.
 */
export function useDraggable(
  initialGridPosition: GridPosition,
  onDragEnd?: (pos: GridPosition) => void,
  onDrag?: (pos: GridPosition, rawOffset?: { x: number, y: number }, continuousPos?: GridPosition) => void,
  scale: number = 1.0,
  onDragStart?: () => void
) {
  const [dragOffset, setDragOffset] = useState<{ x: number, y: number }>({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);

  // We store the grab point in local (scaled) space relative to the module's initial top-left.
  const grabPointLocal = useRef<{ x: number, y: number }>({ x: 0, y: 0 });
  const startScale = useRef<number>(1.0);

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

  const onMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0) return;

    const target = e.target as HTMLElement;
    if (
      target.tagName === 'BUTTON' ||
      target.tagName === 'INPUT' ||
      target.tagName === 'SELECT' ||
      target.closest('.switch') ||
      target.closest('.slider') ||
      target.closest('.custom-select')
    ) {
      return;
    }

    setIsDragging(true);
    startGridPos.current = initialGridPosition;
    startScale.current = scaleRef.current;

    // Calculate where on the module we clicked in local scaled units at the START scale
    const mouseXLocal = e.clientX / startScale.current;
    const mouseYLocal = e.clientY / startScale.current;

    grabPointLocal.current = {
      x: mouseXLocal - (initialGridPosition.gx * GRID_UNIT_PX),
      y: mouseYLocal - (initialGridPosition.gy * GRID_UNIT_PX)
    };

    if (onDragStart) onDragStart();
    e.preventDefault();
  }, [initialGridPosition]);

  useEffect(() => {
    if (!isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      // Calculate where the module top-left should be in local space using START scale
      // This keeps the mouse "stuck" to the same pixel on the module.
      const mouseXLocal = e.clientX / startScale.current;
      const mouseYLocal = e.clientY / startScale.current;

      const targetXLocal = mouseXLocal - grabPointLocal.current.x;
      const targetYLocal = mouseYLocal - grabPointLocal.current.y;

      const startXLocal = startGridPos.current.gx * GRID_UNIT_PX;
      const startYLocal = startGridPos.current.gy * GRID_UNIT_PX;

      const nextOffset = {
        x: targetXLocal - startXLocal,
        y: targetYLocal - startYLocal
      };

      setDragOffset(nextOffset);

      // Logical grid positions should always be relative to the BASE units (non-scaled)
      const continuousGx = targetXLocal / GRID_UNIT_PX;
      const continuousGy = targetYLocal / GRID_UNIT_PX;

      const snappedGx = Math.max(0, Math.round(continuousGx));
      const snappedGy = Math.max(0, Math.round(continuousGy));

      const newPos = { gx: snappedGx, gy: snappedGy };
      const continuousPos = { gx: continuousGx, gy: continuousGy };

      if (onDrag) onDrag(newPos, nextOffset, continuousPos);

      if (newPos.gx !== currentGridPos.current.gx || newPos.gy !== currentGridPos.current.gy) {
        currentGridPos.current = newPos;
      }
    };

    const handleMouseUp = () => {
      setIsDragging(false);
      if (onDragEnd) {
        onDragEnd(currentGridPos.current);
      }
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);

    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging, onDrag, onDragEnd]);

  return {
    dragOffset,
    isDragging,
    onMouseDown,
  };
}

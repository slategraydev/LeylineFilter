// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import { useState, useCallback, useEffect, useRef } from 'react';
import { GRID_UNIT_PX } from '../constants';

export interface GridPosition {
  gx: number;
  gy: number;
}

/**
 * # useDraggable Hook
 * Provides smooth dragging with pixel offsets and scale-aware mouse tracking.
 */
export function useDraggable(
  initialGridPosition: GridPosition,
  onDragEnd?: (pos: GridPosition) => void,
  onDrag?: (pos: GridPosition) => void,
  scale: number = 1.0
) {
  const [dragOffset, setDragOffset] = useState<{ x: number, y: number }>({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);

  const startGridPos = useRef<GridPosition>(initialGridPosition);
  const startMousePos = useRef<{ x: number, y: number }>({ x: 0, y: 0 });
  const currentGridPos = useRef<GridPosition>(initialGridPosition);

  // Use a ref for scale to avoid closure staleness during heavy dragging
  const scaleRef = useRef(scale);
  useEffect(() => {
    scaleRef.current = scale;
  }, [scale]);

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
    startMousePos.current = { x: e.clientX, y: e.clientY };
    e.preventDefault();
  }, [initialGridPosition]);

  useEffect(() => {
    if (!isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      // Delta in screen pixels
      const dxScreen = e.clientX - startMousePos.current.x;
      const dyScreen = e.clientY - startMousePos.current.y;

      // Convert screen delta to local (scaled) space
      const dxLocal = dxScreen / scaleRef.current;
      const dyLocal = dyScreen / scaleRef.current;

      setDragOffset({ x: dxLocal, y: dyLocal });

      // Calculate new logical grid position
      const continuousGx = startGridPos.current.gx + (dxLocal / GRID_UNIT_PX);
      const continuousGy = startGridPos.current.gy + (dyLocal / GRID_UNIT_PX);

      const snappedGx = Math.max(0, Math.round(continuousGx));
      const snappedGy = Math.max(0, Math.round(continuousGy));

      const newPos = { gx: snappedGx, gy: snappedGy };

      if (newPos.gx !== currentGridPos.current.gx || newPos.gy !== currentGridPos.current.gy) {
        currentGridPos.current = newPos;
        if (onDrag) onDrag(newPos);
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

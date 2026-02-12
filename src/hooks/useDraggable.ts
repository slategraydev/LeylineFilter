// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import { useState, useCallback, useEffect, useRef } from 'react';
import { GRID_UNIT_PX } from '../constants';

export interface GridPosition {
  gx: number; // Grid X (units)
  gy: number; // Grid Y (units)
}

/**
 * # useDraggable Hook
 * Provides grid-snapping drag functionality for UI components.
 */
export function useDraggable(
  initialGridPosition: GridPosition,
  onDragEnd?: (pos: GridPosition) => void
) {
  const [gridPosition, setGridPosition] = useState<GridPosition>(initialGridPosition);
  const [isDragging, setIsDragging] = useState(false);

  const startGridPos = useRef<GridPosition>(initialGridPosition);
  const startMousePos = useRef<{ x: number, y: number }>({ x: 0, y: 0 });
  const currentGridPos = useRef<GridPosition>(initialGridPosition);

  useEffect(() => {
    currentGridPos.current = gridPosition;
  }, [gridPosition]);

  useEffect(() => {
    if (!isDragging) {
      setGridPosition(initialGridPosition);
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
    startGridPos.current = currentGridPos.current;
    startMousePos.current = { x: e.clientX, y: e.clientY };
    e.preventDefault();
  }, []);

  useEffect(() => {
    if (!isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      const dx = e.clientX - startMousePos.current.x;
      const dy = e.clientY - startMousePos.current.y;

      // Calculate change in grid units
      const dgx = Math.round(dx / GRID_UNIT_PX);
      const dgy = Math.round(dy / GRID_UNIT_PX);

      // New grid position clamped to 0
      const newGx = Math.max(0, startGridPos.current.gx + dgx);
      const newGy = Math.max(0, startGridPos.current.gy + dgy);

      const newPos = { gx: newGx, gy: newGy };
      currentGridPos.current = newPos; // Update ref immediately
      setGridPosition(newPos);
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
  }, [isDragging, onDragEnd]);

  return {
    gridPosition,
    isDragging,
    onMouseDown,
    setGridPosition
  };
}

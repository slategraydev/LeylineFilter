// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import React from 'react';
import { useDraggable, GridPosition } from '../../hooks/useDraggable';
import { GRID_UNIT_PX } from '../../constants';
import './BaseModule.css';

interface BaseModuleProps {
  id: string;
  initialPosition: GridPosition;
  heightUnits: number;
  onPositionChange: (id: string, pos: GridPosition) => void;
  title: string;
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  onRemove?: () => void;
  children: React.ReactNode;
  hideToggle?: boolean;
  style?: React.CSSProperties;
}

/**
 * # Component Wrapper
 * Provides consistent styling, draggability, and the enable/disable toggle for all audio modules.
 */
export function BaseModule({
  id,
  initialPosition,
  heightUnits,
  onPositionChange,
  title,
  enabled,
  onToggle,
  onRemove,
  children,
  hideToggle = false,
  style,
}: BaseModuleProps) {
  const { gridPosition, isDragging, onMouseDown } = useDraggable(
    initialPosition,
    (newPos) => onPositionChange(id, newPos)
  );

  // Apply absolute positioning and z-index
  const combinedStyle: React.CSSProperties = {
    position: 'absolute',
    left: `${gridPosition.gx * GRID_UNIT_PX}px`,
    top: `${gridPosition.gy * GRID_UNIT_PX}px`,
    height: `${heightUnits * GRID_UNIT_PX}px`,
    zIndex: isDragging ? 100 : 1,
    ...style,
  };

  return (
    <div
      className={`module-card ${enabled ? 'active' : 'inactive'}`}
      style={combinedStyle}
    >
      <div className="module-header" onMouseDown={onMouseDown}>
        <div className="header-left" onMouseDown={(e) => e.stopPropagation()}>
          {!hideToggle && (
            <label className="switch">
              <input
                type="checkbox"
                checked={enabled}
                onChange={(e) => onToggle(e.target.checked)}
              />
              <span className="slider round"></span>
            </label>
          )}
        </div>

        <div className="header-center">
          <h3>{title}</h3>
        </div>

        <div className="header-right" onMouseDown={(e) => e.stopPropagation()}>
          {onRemove && (
            <button
              className="remove-module-btn"
              onClick={onRemove}
              aria-label="Remove Module"
            >
              <svg viewBox="0 0 24 24" width="16" height="16">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          )}
        </div>
      </div>
      <div className="module-content">
        {children}
      </div>
    </div>
  );
}

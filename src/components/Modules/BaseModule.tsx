// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import React, { useLayoutEffect, useRef } from 'react';
import { useDraggable, GridPosition } from '../../hooks/useDraggable';
import { GRID_UNIT_PX } from '../../constants';
import './BaseModule.css';

interface BaseModuleProps {
  id: string;
  initialPosition: GridPosition;
  heightUnits: number;
  scale?: number;
  onPositionChange: (id: string, pos: GridPosition) => void;
  onDrag?: (id: string, pos: GridPosition | null) => void;
  onHeightReport?: (id: string, units: number) => void;
  title: string;
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  onRemove?: () => void;
  children: React.ReactNode;
  hideToggle?: boolean;
  style?: React.CSSProperties;
  isNewlyPlaced?: boolean;
}

/**
 * # Component Wrapper
 * Provides consistent styling, draggability, and the enable/disable toggle for all audio modules.
 */
export function BaseModule({
  id,
  initialPosition,
  heightUnits,
  scale = 1.0,
  onPositionChange,
  onDrag,
  onHeightReport,
  title,
  enabled,
  onToggle,
  onRemove,
  children,
  hideToggle = false,
  style,
  isNewlyPlaced = false,
}: BaseModuleProps) {
  const { dragOffset, isDragging, onMouseDown } = useDraggable(
    initialPosition,
    (newPos) => {
      onPositionChange(id, newPos);
      if (onDrag) onDrag(id, null);
    },
    (newPos) => {
      if (onDrag) onDrag(id, newPos);
    },
    scale
  );

  const moduleRef = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    if (!moduleRef.current || !onHeightReport) return;

    const measure = () => {
      const content = moduleRef.current?.querySelector('.module-content') as HTMLElement;
      if (content) {
        const children = Array.from(content.children) as HTMLElement[];
        let naturalContentHeight = 0;
        if (children.length > 0) {
          const lastChild = children[children.length - 1];
          naturalContentHeight = lastChild.offsetTop + lastChild.offsetHeight + GRID_UNIT_PX;
        } else {
          naturalContentHeight = GRID_UNIT_PX * 2;
        }
        const headerHeight = GRID_UNIT_PX * 4;
        const totalHeight = headerHeight + naturalContentHeight;
        const units = Math.ceil(totalHeight / GRID_UNIT_PX);
        if (units !== heightUnits) {
          onHeightReport(id, units);
        }
      }
    };

    const observer = new ResizeObserver(measure);
    const content = moduleRef.current.querySelector('.module-content');
    if (content) {
      observer.observe(content);
      Array.from(content.children).forEach(child => observer.observe(child));
    }
    measure();
    return () => observer.disconnect();
  }, [id, heightUnits, onHeightReport, children]);

  // Snap instantly while dragging
  const snappedX = initialPosition.gx + Math.round(dragOffset.x / GRID_UNIT_PX);
  const snappedY = initialPosition.gy + Math.round(dragOffset.y / GRID_UNIT_PX);

  const displayX = isDragging ? snappedX * GRID_UNIT_PX : initialPosition.gx * GRID_UNIT_PX;
  const displayY = isDragging ? snappedY * GRID_UNIT_PX : initialPosition.gy * GRID_UNIT_PX;

  const combinedStyle: React.CSSProperties = {
    position: 'absolute',
    left: `${displayX}px`,
    top: `${displayY}px`,
    height: `${heightUnits * GRID_UNIT_PX}px`,
    zIndex: isDragging ? 1000 : 1,
    transition: isDragging ? 'none' : 'left 0.4s cubic-bezier(0.2, 0.8, 0.2, 1), top 0.4s cubic-bezier(0.2, 0.8, 0.2, 1)',
    ...style,
  };

  return (
    <div
      ref={moduleRef}
      className={`module-card ${enabled ? 'active' : 'inactive'} ${isDragging ? 'dragging' : ''} ${isNewlyPlaced ? 'newly-placed' : ''}`}
      style={combinedStyle}
    >
      <div className="module-header" onMouseDown={onMouseDown} style={{ cursor: 'grab' }}>
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

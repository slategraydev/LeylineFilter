// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import React, { useLayoutEffect, useRef } from 'react';
import { useDraggable, GridPosition } from '../../hooks/useDraggable';
import { GRID_UNIT_PX } from '../../constants';
import './BaseModule.css';

interface BaseModuleProps {
  id: string;
  initialPosition: GridPosition;
  heightUnits: number;
  onPositionChange: (id: string, pos: GridPosition) => void;
  onHeightReport?: (id: string, units: number) => void;
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
  onHeightReport,
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

  const moduleRef = useRef<HTMLDivElement>(null);

  // Dynamically measure and report height to the nearest grid unit
  useLayoutEffect(() => {
    if (!moduleRef.current || !onHeightReport) return;

    const measure = () => {
      const content = moduleRef.current?.querySelector('.module-content') as HTMLElement;

      if (content) {
        const children = Array.from(content.children) as HTMLElement[];
        let naturalContentHeight = 0;

        if (children.length > 0) {
          // The natural height of the content is the bottom edge of the last child
          // plus the bottom padding (1 grid unit).
          const lastChild = children[children.length - 1];
          naturalContentHeight = lastChild.offsetTop + lastChild.offsetHeight + GRID_UNIT_PX;
        } else {
          // Empty content still has top and bottom padding
          naturalContentHeight = GRID_UNIT_PX * 2;
        }

        // Total height = Header (4 units) + naturalContentHeight
        const headerHeight = GRID_UNIT_PX * 4;
        const totalHeight = headerHeight + naturalContentHeight;

        // Snap to the nearest grid unit
        const units = Math.ceil(totalHeight / GRID_UNIT_PX);

        if (units !== heightUnits) {
          onHeightReport(id, units);
        }
      }
    };

    // Observe children because the content container itself is stretched by flex: 1
    // and won't change size when its children grow/shrink unless they overflow.
    const observer = new ResizeObserver(measure);
    const content = moduleRef.current.querySelector('.module-content');
    if (content) {
      // Observe the content container for padding/size changes
      observer.observe(content);
      // Also observe every child to catch internal layout changes
      Array.from(content.children).forEach(child => observer.observe(child));
    }

    // Initial measure
    measure();

    return () => observer.disconnect();
  }, [id, heightUnits, onHeightReport, children]); // Re-run when children structure changes
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
      ref={moduleRef}
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

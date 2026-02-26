// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import React, { useLayoutEffect, useRef } from "react";
import { useDraggable, GridPosition } from "../../hooks/useDraggable";
import { GRID_UNIT_PX } from "../../constants";
import "./BaseModule.css";

interface BaseModuleProps {
  id: string;
  initialPosition: GridPosition;
  heightUnits: number;
  widthUnits: number;
  onPositionChange: (id: string, pos: GridPosition) => void;
  onDrag?: (
    id: string,
    pos: GridPosition | null,
    rawOffset?: { x: number; y: number },
    continuousPos?: GridPosition,
  ) => void;
  onHeightReport?: (id: string, units: number) => void;
  onWidthReport?: (id: string, units: number) => void;
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
  widthUnits,
  onPositionChange,
  onDrag,
  onHeightReport,
  onWidthReport,
  title,
  enabled,
  onToggle,
  onRemove,
  children,
  hideToggle = false,
  style,
  isNewlyPlaced = false,
}: BaseModuleProps) {
  const { dragOffset, isDragging, onPointerDown } = useDraggable(
    initialPosition,
    (newPos) => {
      onPositionChange(id, newPos);
      if (onDrag) onDrag(id, null);
    },
    (newPos, rawOffset, continuousPos) => {
      if (onDrag) onDrag(id, newPos, rawOffset, continuousPos);
    },
    1.0, // Scale is now always 1.0
    () => {
      if (onDrag) onDrag(id, initialPosition, { x: 0, y: 0 }, initialPosition);
    },
  );

  const [isResizing, setIsResizing] = React.useState(false);

  const onResizePointerDown = (e: React.PointerEvent) => {
    e.stopPropagation();
    setIsResizing(true);
    if ((e.currentTarget as HTMLElement).setPointerCapture) {
      (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    }
  };

  React.useEffect(() => {
    if (!isResizing) return;

    const handlePointerMove = (e: PointerEvent) => {
      const gridInner = document.querySelector(".grid-inner") as HTMLElement;
      if (!gridInner) return;

      const rect = gridInner.getBoundingClientRect();
      const gridUnit = GRID_UNIT_PX;

      // mouse_local_units = (clientX - gridLeft) / gridUnit
      const mouseXLocal = (e.clientX - rect.left) / gridUnit;
      const mouseYLocal = (e.clientY - rect.top) / gridUnit;

      const newWidthUnits = Math.max(
        10,
        Math.round(mouseXLocal - initialPosition.gx),
      );
      const newHeightUnits = Math.max(
        4,
        Math.round(mouseYLocal - initialPosition.gy),
      );

      if (onWidthReport && newWidthUnits !== widthUnits) {
        onWidthReport(id, newWidthUnits);
      }
      if (onHeightReport && newHeightUnits !== heightUnits) {
        onHeightReport(id, newHeightUnits);
      }
    };

    const handlePointerUp = () => {
      setIsResizing(false);
    };

    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", handlePointerUp);
    return () => {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
    };
  }, [
    isResizing,
    id,
    initialPosition,
    widthUnits,
    heightUnits,
    onWidthReport,
    onHeightReport,
  ]);

  const moduleRef = useRef<HTMLDivElement>(null);

  // Automatic height measurement only if not manually resizing
  useLayoutEffect(() => {
    if (!moduleRef.current || !onHeightReport || isResizing) return;

    const measure = () => {
      const content = moduleRef.current?.querySelector(
        ".module-content",
      ) as HTMLElement;
      if (content) {
        const gridUnit = GRID_UNIT_PX;

        const children = Array.from(content.children) as HTMLElement[];
        const visibleChildren = children.filter((child) => {
          const style = window.getComputedStyle(child);
          return (
            style.display !== "none" &&
            style.visibility !== "hidden" &&
            child.offsetHeight > 0
          );
        });

        let naturalContentHeight = 0;
        if (visibleChildren.length > 0) {
          const lastChild = visibleChildren[visibleChildren.length - 1];
          naturalContentHeight =
            lastChild.offsetTop + lastChild.offsetHeight + gridUnit;
        } else {
          naturalContentHeight = gridUnit * 2;
        }
        const headerHeight = gridUnit * 4;
        const totalHeight = headerHeight + naturalContentHeight;
        const units = Math.ceil(totalHeight / gridUnit);
        if (units !== heightUnits) {
          onHeightReport(id, units);
        }
      }
    };

    const observer = new ResizeObserver(measure);
    const content = moduleRef.current.querySelector(".module-content");
    if (content) {
      observer.observe(content);
      Array.from(content.children).forEach((child) => observer.observe(child));
    }
    measure();
    return () => observer.disconnect();
  }, [id, heightUnits, onHeightReport, isResizing]);

  // Snap instantly while dragging
  const snappedX = initialPosition.gx + Math.round(dragOffset.x / GRID_UNIT_PX);
  const snappedY = initialPosition.gy + Math.round(dragOffset.y / GRID_UNIT_PX);

  const displayX = isDragging ? snappedX : initialPosition.gx;
  const displayY = isDragging ? snappedY : initialPosition.gy;

  const combinedStyle: React.CSSProperties = {
    position: "absolute",
    left: `calc(var(--grid-unit) * ${displayX})`,
    top: `calc(var(--grid-unit) * ${displayY})`,
    width: `calc(var(--grid-unit) * ${widthUnits})`,
    height: `calc(var(--grid-unit) * ${heightUnits})`,
    boxSizing: "border-box",
    zIndex: isDragging || isResizing ? 1000 : 1,
    transition:
      isDragging || isResizing
        ? "none"
        : "left 0.4s cubic-bezier(0.2, 0.8, 0.2, 1), top 0.4s cubic-bezier(0.2, 0.8, 0.2, 1), width 0.4s cubic-bezier(0.2, 0.8, 0.2, 1), height 0.4s cubic-bezier(0.2, 0.8, 0.2, 1)",
    ...style,
  };

  return (
    <div
      ref={moduleRef}
      className={`module-card ${enabled ? "active" : "inactive"} ${isDragging ? "dragging" : ""} ${isNewlyPlaced ? "newly-placed" : ""} ${isResizing ? "resizing" : ""}`}
      style={combinedStyle}
      data-dragging={isDragging}
      onContextMenu={(e) => {
        if (onRemove) {
          e.preventDefault();
          onRemove();
        }
      }}
      onMouseDown={(e) => {
        if (e.button === 1) {
          // Middle click
          e.preventDefault();
          e.stopPropagation();
          onToggle(!enabled);
        }
      }}
    >
      <div
        className="module-header"
        onPointerDown={onPointerDown}
        style={{ cursor: "grab" }}
      >
        <div className="header-left" onPointerDown={(e) => e.stopPropagation()}>
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

        <div
          className="header-right"
          onPointerDown={(e) => e.stopPropagation()}
        >
          {onRemove && (
            <button
              className="remove-module-btn"
              onClick={onRemove}
              aria-label="Remove Module"
            >
              <svg viewBox="0 0 24 24" width="24" height="24">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          )}
        </div>
      </div>
      <div className="module-content">{children}</div>
      <div className="resize-handle" onPointerDown={onResizePointerDown} />
    </div>
  );
}

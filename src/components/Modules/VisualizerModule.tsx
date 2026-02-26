// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import { BaseModule } from "./BaseModule";
import { Visualizer } from "../Visualizer/Visualizer";
import { GridPosition } from "../../types";

interface VisualizerModuleProps {
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
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  onRemove?: () => void;
  isRunning: boolean;
  spectrum: number[];
  tonality: number[];
  style?: React.CSSProperties;
  isNewlyPlaced?: boolean;
}

/**
 * # Visualizer Module
 * Wraps the Visualizer component in a BaseModule to fit in the module grid.
 */
export function VisualizerModule({
  id,
  initialPosition,
  heightUnits,
  widthUnits,
  onPositionChange,
  onDrag,
  onHeightReport,
  onWidthReport,
  enabled,
  onToggle,
  onRemove,
  isRunning,
  spectrum,
  tonality,
  style,
  isNewlyPlaced,
}: VisualizerModuleProps) {
  return (
    <BaseModule
      id={id}
      initialPosition={initialPosition}
      heightUnits={heightUnits}
      widthUnits={widthUnits}
      onPositionChange={onPositionChange}
      onDrag={onDrag}
      onHeightReport={onHeightReport}
      onWidthReport={onWidthReport}
      title="Spectrum Analyzer"
      enabled={enabled}
      onToggle={onToggle}
      onRemove={onRemove}
      hideToggle={false} // Allow disabling it if desired
      style={style}
      isNewlyPlaced={isNewlyPlaced}
    >
      <div
        style={{ flex: 1, minHeight: "100px", width: "100%", marginTop: "4px" }}
      >
        <Visualizer
          isRunning={isRunning && enabled}
          spectrum={spectrum}
          tonality={tonality}
        />
      </div>
    </BaseModule>
  );
}

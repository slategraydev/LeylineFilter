// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import { BaseModule } from './BaseModule';
import { Visualizer } from '../Visualizer/Visualizer';
import { GridPosition } from '../../hooks/useDraggable';

interface VisualizerModuleProps {
  id: string;
  initialPosition: GridPosition;
  heightUnits: number;
  onPositionChange: (id: string, pos: GridPosition) => void;
  onHeightReport?: (id: string, units: number) => void;
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  onRemove?: () => void;
  isRunning: boolean;
  spectrum: number[];
  tonality: number[];
  style?: React.CSSProperties;
}

/**
 * # Visualizer Module
 * Wraps the Visualizer component in a BaseModule to fit in the module grid.
 */
export function VisualizerModule({
  id,
  initialPosition,
  heightUnits,
  onPositionChange,
  onHeightReport,
  enabled,
  onToggle,
  onRemove,
  isRunning,
  spectrum,
  tonality,
  style,
}: VisualizerModuleProps) {
  return (
    <BaseModule
      id={id}
      initialPosition={initialPosition}
      heightUnits={heightUnits}
      onPositionChange={onPositionChange}
      onHeightReport={onHeightReport}
      title="Spectrum Analyzer"
      enabled={enabled}
      onToggle={onToggle}
      onRemove={onRemove}
      hideToggle={false} // Allow disabling it if desired
      style={style}
    >
      <div style={{ flex: 1, minHeight: '120px', marginTop: '4px' }}>
        <Visualizer
          isRunning={isRunning && enabled}
          spectrum={spectrum}
          tonality={tonality}
        />
      </div>
    </BaseModule>
  );
}

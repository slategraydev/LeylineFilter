// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import { BaseModule } from './BaseModule';
import { Visualizer } from '../Visualizer/Visualizer';
import { GridPosition } from '../../hooks/useDraggable';

interface VisualizerModuleProps {
  id: string;
  initialPosition: GridPosition;
  heightUnits: number;
  onPositionChange: (id: string, pos: GridPosition) => void;
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
      title="Spectrum Analyzer"
      enabled={enabled}
      onToggle={onToggle}
      onRemove={onRemove}
      hideToggle={false} // Allow disabling it if desired
      style={style}
    >
      <div style={{ height: '180px', marginTop: '4px' }}>
        <Visualizer
          isRunning={isRunning && enabled}
          spectrum={spectrum}
          tonality={tonality}
        />
      </div>
    </BaseModule>
  );
}

// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import { BaseModule } from './BaseModule';
import { Visualizer } from '../Visualizer/Visualizer';

interface VisualizerModuleProps {
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  isRunning: boolean;
  spectrum: number[];
  tonality: number[];
}

/**
 * # Visualizer Module
 * Wraps the Visualizer component in a BaseModule to fit in the module grid.
 */
export function VisualizerModule({
  enabled,
  onToggle,
  isRunning,
  spectrum,
  tonality
}: VisualizerModuleProps) {
  return (
    <BaseModule
      title="Spectrum Analyzer"
      enabled={enabled}
      onToggle={onToggle}
      hideToggle={false} // Allow disabling it if desired
    >
      <div style={{ height: '140px', marginTop: '10px' }}>
        <Visualizer
          isRunning={isRunning && enabled}
          spectrum={spectrum}
          tonality={tonality}
        />
      </div>
    </BaseModule>
  );
}

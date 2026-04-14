import { ExpanderConfig, GridPosition } from '../../types';
import { BaseModule } from './BaseModule';

interface ExpanderModuleProps {
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
  config: ExpanderConfig;
  onChange: (config: ExpanderConfig) => void;
  onRemove?: () => void;
  style?: React.CSSProperties;
  isNewlyPlaced?: boolean;
}

export function ExpanderModule({
  id,
  initialPosition,
  heightUnits,
  widthUnits,
  onPositionChange,
  onDrag,
  onHeightReport,
  onWidthReport,
  config,
  onChange,
  onRemove,
  style,
  isNewlyPlaced,
}: ExpanderModuleProps) {
  const updateConfig = (updates: Partial<ExpanderConfig>) => {
    onChange({ ...config, ...updates });
  };

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
      title="Noise Expander"
      enabled={config.enabled}
      onToggle={(enabled) => updateConfig({ enabled })}
      onRemove={onRemove}
      style={style}
      isNewlyPlaced={isNewlyPlaced}
    >
      <p className="module-description">
        Reduces low-level noise by attenuating signals below the threshold.
      </p>
      <div className="control-group">
        <label>
          Threshold <span>{(20 * Math.log10(config.threshold)).toFixed(1)} dB</span>
          <input
            type="range"
            min="0.0001"
            max="0.5"
            step="0.0001"
            value={config.threshold}
            disabled={!config.enabled}
            onChange={(e) => updateConfig({ threshold: parseFloat(e.target.value) })}
          />
        </label>
      </div>
      <div className="control-group">
        <label>
          Ratio <span>{config.ratio.toFixed(1)}:1</span>
          <input
            type="range"
            min="1.0"
            max="10.0"
            step="0.1"
            value={config.ratio}
            disabled={!config.enabled}
            onChange={(e) => updateConfig({ ratio: parseFloat(e.target.value) })}
          />
        </label>
      </div>
      <div className="control-group">
        <label>
          Attack <span>{config.attack_ms.toFixed(1)} ms</span>
          <input
            type="range"
            min="0.1"
            max="100"
            step="0.1"
            value={config.attack_ms}
            disabled={!config.enabled}
            onChange={(e) => updateConfig({ attack_ms: parseFloat(e.target.value) })}
          />
        </label>
      </div>
      <div className="control-group">
        <label>
          Release <span>{config.release_ms.toFixed(0)} ms</span>
          <input
            type="range"
            min="10"
            max="1000"
            step="1"
            value={config.release_ms}
            disabled={!config.enabled}
            onChange={(e) => updateConfig({ release_ms: parseFloat(e.target.value) })}
          />
        </label>
      </div>
    </BaseModule>
  );
}

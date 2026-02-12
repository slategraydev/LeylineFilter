import { ExpanderConfig } from "../../types";
import { BaseModule } from "./BaseModule";
import { GridPosition } from "../../hooks/useDraggable";

interface ExpanderModuleProps {
  id: string;
  initialPosition: GridPosition;
  heightUnits: number;
  onPositionChange: (id: string, pos: GridPosition) => void;
  config: ExpanderConfig;
  onChange: (config: ExpanderConfig) => void;
  onRemove?: () => void;
  style?: React.CSSProperties;
}

export function ExpanderModule({
  id,
  initialPosition,
  heightUnits,
  onPositionChange,
  config,
  onChange,
  onRemove,
  style,
}: ExpanderModuleProps) {
  const updateConfig = (updates: Partial<ExpanderConfig>) => {
    onChange({ ...config, ...updates });
  };

  return (
    <BaseModule
      id={id}
      initialPosition={initialPosition}
      heightUnits={heightUnits}
      onPositionChange={onPositionChange}
      title="Noise Expander"
      enabled={config.enabled}
      onToggle={(enabled) => updateConfig({ enabled })}
      onRemove={onRemove}
      style={style}
    >
      <div className="control-group">
        <label>
          Threshold: {Math.round(config.threshold * 1000)}
          <input
            type="range"
            min="0.001"
            max="0.5"
            step="0.001"
            value={config.threshold}
            disabled={!config.enabled}
            onChange={(e) =>
              updateConfig({ threshold: parseFloat(e.target.value) })
            }
          />
        </label>
      </div>
      <div className="control-group">
        <label>
          Ratio: {config.ratio.toFixed(1)}:1
          <input
            type="range"
            min="1.0"
            max="10.0"
            step="0.1"
            value={config.ratio}
            disabled={!config.enabled}
            onChange={(e) =>
              updateConfig({ ratio: parseFloat(e.target.value) })
            }
          />
        </label>
      </div>
      <div className="control-group">
        <label>
          Attack: {config.attack_ms.toFixed(0)} ms
          <input
            type="range"
            min="0.1"
            max="200"
            step="0.1"
            value={config.attack_ms}
            disabled={!config.enabled}
            onChange={(e) =>
              updateConfig({ attack_ms: parseFloat(e.target.value) })
            }
          />
        </label>
      </div>
      <div className="control-group">
        <label>
          Release: {config.release_ms.toFixed(0)} ms
          <input
            type="range"
            min="10"
            max="1000"
            step="1"
            value={config.release_ms}
            disabled={!config.enabled}
            onChange={(e) =>
              updateConfig({ release_ms: parseFloat(e.target.value) })
            }
          />
        </label>
      </div>
    </BaseModule>
  );
}

import { CompressorConfig } from "../../types";
import { BaseModule } from "./BaseModule";
import { GridPosition } from "../../hooks/useDraggable";

interface CompressorModuleProps {
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
  config: CompressorConfig;
  onChange: (config: CompressorConfig) => void;
  onRemove?: () => void;
  style?: React.CSSProperties;
  isNewlyPlaced?: boolean;
}

export function CompressorModule({
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
}: CompressorModuleProps) {
  const updateConfig = (updates: Partial<CompressorConfig>) => {
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
      title="Dynamic Compressor"
      enabled={config.enabled}
      onToggle={(enabled) => updateConfig({ enabled })}
      onRemove={onRemove}
      style={style}
      isNewlyPlaced={isNewlyPlaced}
    >
      <p className="module-description">
        Evens out dynamic peaks for a more professional and consistent sound.
      </p>
      <div className="control-group">
        <label>
          Threshold <span>{config.threshold_db.toFixed(1)} dB</span>
          <input
            type="range"
            min="-60"
            max="0"
            step="0.1"
            value={config.threshold_db}
            disabled={!config.enabled}
            onChange={(e) =>
              updateConfig({ threshold_db: parseFloat(e.target.value) })
            }
          />
        </label>
      </div>
      <div className="control-group">
        <label>
          Ratio <span>{config.ratio.toFixed(1)}:1</span>
          <input
            type="range"
            min="1.0"
            max="20.0"
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
          Knee <span>{config.knee_db.toFixed(1)} dB</span>
          <input
            type="range"
            min="0"
            max="24"
            step="0.1"
            value={config.knee_db}
            disabled={!config.enabled}
            onChange={(e) =>
              updateConfig({ knee_db: parseFloat(e.target.value) })
            }
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
            onChange={(e) =>
              updateConfig({ attack_ms: parseFloat(e.target.value) })
            }
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
            onChange={(e) =>
              updateConfig({ release_ms: parseFloat(e.target.value) })
            }
          />
        </label>
      </div>
      <div className="control-group">
        <label>
          Makeup <span>{config.makeup_gain_db.toFixed(1)} dB</span>
          <input
            type="range"
            min="0"
            max="24"
            step="0.1"
            value={config.makeup_gain_db}
            disabled={!config.enabled}
            onChange={(e) =>
              updateConfig({ makeup_gain_db: parseFloat(e.target.value) })
            }
          />
        </label>
      </div>
    </BaseModule>
  );
}

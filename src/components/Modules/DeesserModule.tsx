import { DeesserConfig } from "../../types";
import { BaseModule } from "./BaseModule";
import { GridPosition } from "../../hooks/useDraggable";

interface DeesserModuleProps {
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
  config: DeesserConfig;
  onChange: (config: DeesserConfig) => void;
  onRemove?: () => void;
  style?: React.CSSProperties;
  isNewlyPlaced?: boolean;
}

export function DeesserModule({
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
}: DeesserModuleProps) {
  const updateConfig = (updates: Partial<DeesserConfig>) => {
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
      title="Vocal De-Esser"
      enabled={config.enabled}
      onToggle={(enabled) => updateConfig({ enabled })}
      onRemove={onRemove}
      style={style}
      isNewlyPlaced={isNewlyPlaced}
    >
      <p className="module-description">
        Transparently reduces harsh sibilant frequencies in vocals.
      </p>
      <div className="control-group">
        <label>
          Threshold <span>{config.threshold_db.toFixed(1)} dB</span>
        </label>
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
      </div>
      <div className="control-group">
        <label>
          Frequency <span>{Math.round(config.frequency)} Hz</span>
        </label>
        <input
          type="range"
          min="1000"
          max="12000"
          step="100"
          value={config.frequency}
          disabled={!config.enabled}
          onChange={(e) =>
            updateConfig({ frequency: parseFloat(e.target.value) })
          }
        />
      </div>
      <div className="control-group">
        <label>
          Ratio <span>{config.ratio.toFixed(1)}:1</span>
        </label>
        <input
          type="range"
          min="1.0"
          max="20.0"
          step="0.1"
          value={config.ratio}
          disabled={!config.enabled}
          onChange={(e) => updateConfig({ ratio: parseFloat(e.target.value) })}
        />
      </div>
    </BaseModule>
  );
}

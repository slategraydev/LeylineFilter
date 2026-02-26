import { SaturationConfig, GridPosition } from "../../types";
import { BaseModule } from "./BaseModule";

interface SaturationModuleProps {
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
  config: SaturationConfig;
  onChange: (config: SaturationConfig) => void;
  onRemove?: () => void;
  style?: React.CSSProperties;
  isNewlyPlaced?: boolean;
}

export function SaturationModule({
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
}: SaturationModuleProps) {
  const updateConfig = (updates: Partial<SaturationConfig>) => {
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
      title="Tube Saturation"
      enabled={config.enabled}
      onToggle={(enabled) => updateConfig({ enabled })}
      onRemove={onRemove}
      style={style}
      isNewlyPlaced={isNewlyPlaced}
    >
      <p className="module-description">
        Adds warm harmonic character and vintage analog weight.
      </p>
      <div className="control-group">
        <label>
          Drive <span>{config.drive.toFixed(1)}x</span>
          <input
            type="range"
            min="1.0"
            max="10.0"
            step="0.1"
            value={config.drive}
            disabled={!config.enabled}
            onChange={(e) =>
              updateConfig({ drive: parseFloat(e.target.value) })
            }
          />
        </label>
      </div>
      <div className="control-group">
        <label>
          Tilt (EQ Bias) <span>{config.tilt.toFixed(1)} dB</span>
          <input
            type="range"
            min="-12.0"
            max="12.0"
            step="0.1"
            value={config.tilt}
            disabled={!config.enabled}
            onChange={(e) => updateConfig({ tilt: parseFloat(e.target.value) })}
          />
        </label>
      </div>
      <div className="control-group">
        <label>
          Mix <span>{Math.round(config.mix * 100)}%</span>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={config.mix}
            disabled={!config.enabled}
            onChange={(e) => updateConfig({ mix: parseFloat(e.target.value) })}
          />
        </label>
      </div>
    </BaseModule>
  );
}

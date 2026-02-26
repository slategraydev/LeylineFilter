import { FilterConfig } from "../../types";
import { BaseModule } from "./BaseModule";
import { GridPosition } from "../../hooks/useDraggable";

interface FilterModuleProps {
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
  config: FilterConfig;
  onChange: (config: FilterConfig) => void;
  onRemove?: () => void;
  style?: React.CSSProperties;
  isNewlyPlaced?: boolean;
}

export function FilterModule({
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
}: FilterModuleProps) {
  const updateConfig = (updates: Partial<FilterConfig>) => {
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
      title="Audio Filter"
      enabled={config.enabled}
      onToggle={(enabled) => updateConfig({ enabled })}
      onRemove={onRemove}
      style={style}
      isNewlyPlaced={isNewlyPlaced}
    >
      <p className="module-description">
        Applies frequency-based attenuation to clear up the signal.
      </p>
      <div className="control-group">
        <label>
          Type
          <select
            className="custom-select"
            value={config.filter_type}
            disabled={!config.enabled}
            onChange={(e) =>
              updateConfig({
                filter_type: e.target.value as FilterConfig["filter_type"],
              })
            }
          >
            <option value="LPF">Low Pass</option>
            <option value="HPF">High Pass</option>
            <option value="BPF">Band Pass</option>
            <option value="Notch">Notch</option>
          </select>
        </label>
      </div>
      <div className="control-group">
        <label>
          Frequency <span>{Math.round(config.frequency)} Hz</span>
        </label>
        <input
          type="range"
          min="20"
          max="20000"
          step="1"
          value={config.frequency}
          disabled={!config.enabled}
          onChange={(e) =>
            updateConfig({ frequency: parseFloat(e.target.value) })
          }
        />
      </div>
      <div className="control-group">
        <label>
          Q <span>{config.q.toFixed(2)}</span>
        </label>
        <input
          type="range"
          min="0.1"
          max="10.0"
          step="0.01"
          value={config.q}
          disabled={!config.enabled}
          onChange={(e) => updateConfig({ q: parseFloat(e.target.value) })}
        />
      </div>
    </BaseModule>
  );
}

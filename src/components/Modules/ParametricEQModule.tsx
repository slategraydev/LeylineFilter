import { ParametricEQConfig, EQBandConfig } from "../../types";
import { BaseModule } from "./BaseModule";
import { GridPosition } from "../../hooks/useDraggable";
import "./ParametricEQModule.css";

interface ParametricEQModuleProps {
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
  config: ParametricEQConfig;
  onChange: (config: ParametricEQConfig) => void;
  onRemove?: () => void;
  style?: React.CSSProperties;
  isNewlyPlaced?: boolean;
}

export function ParametricEQModule({
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
}: ParametricEQModuleProps) {
  const updateBand = (index: number, updates: Partial<EQBandConfig>) => {
    const newBands = [...config.bands];
    newBands[index] = { ...newBands[index], ...updates };
    onChange({ ...config, bands: newBands });
  };

  const getBandName = (index: number, type: string) => {
    if (type === "LowShelf") return "Low Shelf";
    if (type === "HighShelf") return "High Shelf";
    return `Mid ${index}`;
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
      title="Parametric EQ"
      enabled={config.enabled}
      onToggle={(enabled) => onChange({ ...config, enabled })}
      onRemove={onRemove}
      style={style}
      isNewlyPlaced={isNewlyPlaced}
    >
      <p className="module-description">
        Advanced multi-band frequency control for precise tonal shaping.
      </p>
      <div className="eq-bands">
        {config.bands.map((band, i) => (
          <div
            key={i}
            className={`eq-band ${!band.enabled ? "band-disabled" : ""}`}
          >
            <div className="band-header">
              <span className="band-name">
                {getBandName(i, band.filter_type)}
              </span>
              <input
                type="checkbox"
                checked={band.enabled}
                onChange={(e) => updateBand(i, { enabled: e.target.checked })}
              />
            </div>

            <div className="control-group">
              <label>
                Freq <span>{Math.round(band.frequency)} Hz</span>
              </label>
              <input
                type="range"
                min="20"
                max="20000"
                step="1"
                value={band.frequency}
                disabled={!config.enabled || !band.enabled}
                onChange={(e) =>
                  updateBand(i, { frequency: parseFloat(e.target.value) })
                }
              />
            </div>

            <div className="control-group">
              <label>
                Gain <span>{band.gain_db.toFixed(1)} dB</span>
              </label>
              <input
                type="range"
                min="-24"
                max="24"
                step="0.1"
                value={band.gain_db}
                disabled={!config.enabled || !band.enabled}
                onChange={(e) =>
                  updateBand(i, { gain_db: parseFloat(e.target.value) })
                }
              />
            </div>

            <div className="control-group">
              <label>
                Q <span>{band.q.toFixed(2)}</span>
              </label>
              <input
                type="range"
                min="0.1"
                max="10.0"
                step="0.01"
                value={band.q}
                disabled={!config.enabled || !band.enabled}
                onChange={(e) =>
                  updateBand(i, { q: parseFloat(e.target.value) })
                }
              />
            </div>
          </div>
        ))}
      </div>
    </BaseModule>
  );
}

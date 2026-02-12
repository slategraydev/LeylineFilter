import React from "react";
import { GainConfig } from "../../types";
import { BaseModule } from "./BaseModule";
import { GridPosition } from "../../hooks/useDraggable";

interface Props {
  id: string;
  initialPosition: GridPosition;
  heightUnits: number;
  onPositionChange: (id: string, pos: GridPosition) => void;
  onHeightReport?: (id: string, units: number) => void;
  config: GainConfig;
  onChange: (config: GainConfig) => void;
  onRemove?: () => void;
  style?: React.CSSProperties;
}

export const GainModule: React.FC<Props> = ({
  id,
  initialPosition,
  heightUnits,
  onPositionChange,
  onHeightReport,
  config,
  onChange,
  onRemove,
  style,
}) => {
  const updateConfig = (updates: Partial<GainConfig>) => {
    onChange({ ...config, ...updates });
  };

  return (
    <BaseModule
      id={id}
      initialPosition={initialPosition}
      heightUnits={heightUnits}
      onPositionChange={onPositionChange}
      onHeightReport={onHeightReport}
      title="Master Gain"
      enabled={config.enabled}
      onToggle={(enabled) => updateConfig({ enabled })}
      onRemove={onRemove}
      style={style}
    >
      <div className="control-group">
        <label>
          Gain: <span>{config.gain_db.toFixed(1)} dB</span>
          <input
            type="range"
            min="-30"
            max="30"
            step="0.1"
            value={config.gain_db}
            disabled={!config.enabled}
            onChange={(e) =>
              updateConfig({ gain_db: parseFloat(e.target.value) })
            }
          />
        </label>
      </div>
    </BaseModule>
  );
};

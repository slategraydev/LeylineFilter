import React from "react";
import { GainConfig } from "../../types";
import { BaseModule } from "./BaseModule";

interface Props {
  config: GainConfig;
  onChange: (config: GainConfig) => void;
}

export const GainModule: React.FC<Props> = ({ config, onChange }) => {
  const updateConfig = (updates: Partial<GainConfig>) => {
    onChange({ ...config, ...updates });
  };

  return (
    <BaseModule
      title="Master Gain"
      enabled={config.enabled}
      onToggle={(enabled) => updateConfig({ enabled })}
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

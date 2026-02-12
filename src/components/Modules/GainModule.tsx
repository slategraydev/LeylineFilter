import React from "react";
import { GainConfig } from "../../types";
import "./BaseModule.css";

interface Props {
  config: GainConfig;
  onChange: (config: GainConfig) => void;
}

export const GainModule: React.FC<Props> = ({ config, onChange }) => {
  const handleGainChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange({ ...config, gain_db: parseFloat(e.target.value) });
  };

  const toggleEnabled = () => {
    onChange({ ...config, enabled: !config.enabled });
  };

  return (
    <div className={`module-card ${!config.enabled ? "disabled" : ""}`}>
      <div className="module-header">
        <h3>Gain</h3>
        <button
          className={`toggle-btn ${config.enabled ? "active" : ""}`}
          onClick={toggleEnabled}
        >
          {config.enabled ? "ON" : "BYPASS"}
        </button>
      </div>

      <div className="module-content">
        <div className="control-group">
          <label>
            Gain: <span>{config.gain_db.toFixed(1)} dB</span>
          </label>
          <input
            type="range"
            min="-30"
            max="30"
            step="0.1"
            value={config.gain_db}
            onChange={handleGainChange}
          />
        </div>
      </div>
    </div>
  );
};

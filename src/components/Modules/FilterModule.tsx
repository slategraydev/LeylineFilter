import { FilterConfig } from "../../types";
import { BaseModule } from "./BaseModule";

interface FilterModuleProps {
  config: FilterConfig;
  onChange: (config: FilterConfig) => void;
}

export function FilterModule({ config, onChange }: FilterModuleProps) {
  const updateConfig = (updates: Partial<FilterConfig>) => {
    onChange({ ...config, ...updates });
  };

  return (
    <BaseModule
      title="Audio Filter"
      enabled={config.enabled}
      onToggle={(enabled) => updateConfig({ enabled })}
    >
      <div className="control-group">
        <label>
          Type
          <select
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
          Frequency: {Math.round(config.frequency)} Hz
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
        </label>
      </div>
      <div className="control-group">
        <label>
          Q: {config.q.toFixed(2)}
          <input
            type="range"
            min="0.1"
            max="10.0"
            step="0.01"
            value={config.q}
            disabled={!config.enabled}
            onChange={(e) => updateConfig({ q: parseFloat(e.target.value) })}
          />
        </label>
      </div>
    </BaseModule>
  );
}

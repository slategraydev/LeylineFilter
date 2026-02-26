import "./EngineControls.css";

interface EngineControlsProps {
  isRunning: boolean;
  isMonitoring: boolean;
  inputDevices: string[];
  outputDevices: string[];
  selectedInput: string;
  selectedOutput: string;
  onInputChange: (device: string) => void;
  onOutputChange: (device: string) => void;
  onToggleMonitoring: (enabled: boolean) => void;
}

export function EngineControls({
  isMonitoring,
  inputDevices,
  outputDevices,
  selectedInput,
  selectedOutput,
  onInputChange,
  onOutputChange,
  onToggleMonitoring,
}: EngineControlsProps) {
  return (
    <div className="engine-controls">
      <div className="device-selectors">
        <div className="selector">
          <label>
            Input
            <select
              value={selectedInput}
              onChange={(e) => onInputChange(e.target.value)}
            >
              {inputDevices.map((d) => (
                <option key={d} value={d}>
                  {d}
                </option>
              ))}
            </select>
          </label>
        </div>
        <div className="selector">
          <label>
            Output
            <select
              value={selectedOutput}
              onChange={(e) => onOutputChange(e.target.value)}
            >
              {outputDevices.map((d) => (
                <option key={d} value={d}>
                  {d}
                </option>
              ))}
            </select>
          </label>
        </div>
      </div>

      <div className="monitoring-toggle">
        <label className="monitor-label">
          Monitor Processed Audio
          <label className="switch">
            <input
              type="checkbox"
              checked={isMonitoring}
              onChange={(e) => onToggleMonitoring(e.target.checked)}
            />
            <span className="slider round"></span>
          </label>
        </label>
      </div>
    </div>
  );
}

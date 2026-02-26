import "./EngineControls.css";

interface EngineControlsProps {
  isRunning: boolean;
  inputDevices: string[];
  outputDevices: string[];
  selectedInput: string;
  selectedOutput: string;
  selectedMonitor: string;
  onInputChange: (device: string) => void;
  onOutputChange: (device: string) => void;
  onMonitorChange: (device: string) => void;
}

export function EngineControls({
  inputDevices,
  outputDevices,
  selectedInput,
  selectedOutput,
  selectedMonitor,
  onInputChange,
  onOutputChange,
  onMonitorChange,
}: EngineControlsProps) {
  return (
    <div className="engine-controls">
      <div className="device-selectors">
        <div className="selector">
          <label>
            Microphone (In)
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
            Destination (Out)
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
        <div className="selector">
          <label>
            Headphones (Monitor)
            <select
              value={selectedMonitor}
              onChange={(e) => onMonitorChange(e.target.value)}
            >
              <option value="None">None</option>
              {outputDevices.map((d) => (
                <option key={d} value={d}>
                  {d}
                </option>
              ))}
            </select>
          </label>
        </div>
      </div>
    </div>
  );
}

interface EngineControlsProps {
  isRunning: boolean;
  inputDevices: string[];
  outputDevices: string[];
  selectedInput: string;
  selectedOutput: string;
  onInputChange: (device: string) => void;
  onOutputChange: (device: string) => void;
  onToggle: () => void;
}

export function EngineControls({
  isRunning,
  inputDevices,
  outputDevices,
  selectedInput,
  selectedOutput,
  onInputChange,
  onOutputChange,
  onToggle,
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
              disabled={isRunning}
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
              disabled={isRunning}
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
    </div>
  );
}

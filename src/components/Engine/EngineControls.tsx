// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ENGINE CONTROLS
// Transport controls and global engine state toggles.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

import './EngineControls.css';

interface EngineControlsProps {
  isRunning: boolean;
  inputDevices: string[];
  outputDevices: string[];
  selectedInput: string;
  selectedOutput: string;
  isMonitoring: boolean;
  onInputChange: (device: string) => void;
  onOutputChange: (device: string) => void;
  onMonitoringChange: (enabled: boolean) => void;
}

export function EngineControls({
  inputDevices,
  outputDevices,
  selectedInput,
  selectedOutput,
  isMonitoring,
  onInputChange,
  onOutputChange,
  onMonitoringChange,
}: EngineControlsProps) {
  return (
    <div className="engine-controls">
      <div className="device-selectors">
        <div className="selector">
          <label>
            Input
            <select value={selectedInput} onChange={(e) => onInputChange(e.target.value)}>
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
            <select value={selectedOutput} onChange={(e) => onOutputChange(e.target.value)}>
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
        <label className="checkbox-container monitor-label">
          Monitor
          <div style={{ display: 'flex', alignItems: 'center' }}>
            <input
              type="checkbox"
              checked={isMonitoring}
              onChange={(e) => onMonitoringChange(e.target.checked)}
            />
            <span className="checkmark"></span>
          </div>
        </label>
      </div>
    </div>
  );
}

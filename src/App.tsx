import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [isRunning, setIsRunning] = useState(false);
  const [inputDevices, setInputDevices] = useState<string[]>([]);
  const [outputDevices, setOutputDevices] = useState<string[]>([]);
  const [selectedInput, setSelectedInput] = useState<string>("Default");
  const [selectedOutput, setSelectedOutput] = useState<string>("Default");
  const [metrics, setMetrics] = useState({
    latency_ms: 0,
    cpu_usage: 0,
  });

  // Load initial config from localStorage or use defaults (80 threshold, 2.0 ratio)
  const [expanderConfig, setExpanderConfig] = useState(() => {
    const saved = localStorage.getItem("expander_config");
    if (saved) {
      try {
        return JSON.parse(saved);
      } catch (e) {
        console.error("Failed to parse saved config", e);
      }
    }
    return {
      enabled: true,
      threshold: 0.08,
      ratio: 2.0,
    };
  });

  useEffect(() => {
    // Fetch available devices
    invoke<string[]>("get_input_devices").then((d) =>
      setInputDevices(["Default", ...d]),
    );
    invoke<string[]>("get_output_devices").then((d) =>
      setOutputDevices(["Default", ...d]),
    );

    const interval = setInterval(async () => {
      try {
        const m = await invoke<{
          latency_ms: number;
          cpu_usage: number;
        }>("get_metrics");
        setMetrics(m);
      } catch (e) {
        // Silently fail if backend not ready
      }
    }, 500);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    // Save to localStorage whenever config changes
    localStorage.setItem("expander_config", JSON.stringify(expanderConfig));

    // Send as enum variant Expander with adjacent tagging
    invoke("update_config", {
      config: {
        type: "Expander",
        data: expanderConfig,
      },
    });
  }, [expanderConfig]);

  async function toggleEngine() {
    if (isRunning) {
      await invoke("stop_engine");
      setIsRunning(false);
    } else {
      await invoke("start_engine", {
        input_device: selectedInput,
        output_device: selectedOutput,
      });
      setIsRunning(true);
    }
  }

  return (
    <div className="container">
      <header>
        <h1>
          LeylineFilter <span>Modular Audio Processor</span>
        </h1>
      </header>

      <main>
        <div className="control-panel">
          <div className="section">
            <label>Input Device</label>
            <select
              value={selectedInput}
              onChange={(e) => setSelectedInput(e.target.value)}
              disabled={isRunning}
              className="device-select"
            >
              {inputDevices.map((d) => (
                <option key={d} value={d}>
                  {d}
                </option>
              ))}
            </select>
          </div>

          <div className="section">
            <label>Output Device</label>
            <select
              value={selectedOutput}
              onChange={(e) => setSelectedOutput(e.target.value)}
              disabled={isRunning}
              className="device-select"
            >
              {outputDevices.map((d) => (
                <option key={d} value={d}>
                  {d}
                </option>
              ))}
            </select>
          </div>

          <div className="section">
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={expanderConfig.enabled}
                onChange={(e) =>
                  setExpanderConfig({
                    ...expanderConfig,
                    enabled: e.target.checked,
                  })
                }
              />
              <span>Enable Noise Expander</span>
            </label>
          </div>

          {expanderConfig.enabled && (
            <>
              <div className="section">
                <label>
                  Expander Threshold:{" "}
                  {Math.round(expanderConfig.threshold * 1000)}
                </label>
                <input
                  type="range"
                  min="0.001"
                  max="0.5"
                  step="0.001"
                  value={expanderConfig.threshold}
                  onChange={(e) =>
                    setExpanderConfig({
                      ...expanderConfig,
                      threshold: parseFloat(e.target.value),
                    })
                  }
                />
              </div>
              <div className="section">
                <label>
                  Expander Ratio: {expanderConfig.ratio.toFixed(1)}:1
                </label>
                <input
                  type="range"
                  min="1.0"
                  max="10.0"
                  step="0.1"
                  value={expanderConfig.ratio}
                  onChange={(e) =>
                    setExpanderConfig({
                      ...expanderConfig,
                      ratio: parseFloat(e.target.value),
                    })
                  }
                />
              </div>
            </>
          )}

          <div className="actions">
            <button
              className={`btn-primary ${isRunning ? "stop" : "start"}`}
              onClick={toggleEngine}
            >
              {isRunning ? "STOP ENGINE" : "START ENGINE"}
            </button>
          </div>
        </div>

        <div className="visualizer-container">
          <div className="wave-placeholder">
            {[...Array(5)].map((_, i) => (
              <div
                key={i}
                className="wave-bar"
                style={{
                  animationPlayState: isRunning ? "running" : "paused",
                }}
              ></div>
            ))}
          </div>
          <div className="status-overlay">
            {isRunning ? "Engine Active" : "Engine Idle"}
          </div>
        </div>
      </main>

      <footer>
        <div className="metric">
          Latency: <span>{metrics.latency_ms}ms</span>
        </div>
        <div className="metric">
          CPU: <span>{metrics.cpu_usage}%</span>
        </div>
        <div className="metric">
          Backend: <span>Rust (Modular)</span>
        </div>
      </footer>
    </div>
  );
}

export default App;

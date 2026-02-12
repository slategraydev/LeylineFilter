import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useEngine } from "./hooks/useEngine";
import { ExpanderModule } from "./components/Modules/ExpanderModule";
import { RNNoiseModule } from "./components/Modules/RNNoiseModule";
import { EngineControls } from "./components/Engine/EngineControls";
import { Visualizer } from "./components/Visualizer/Visualizer";
import { ExpanderConfig, RNNoiseConfig } from "./types";
import "./App.css";

function App() {
  const {
    isRunning,
    inputDevices,
    outputDevices,
    metrics,
    startEngine,
    stopEngine,
  } = useEngine();

  const [selectedInput, setSelectedInput] = useState<string>("Default");
  const [selectedOutput, setSelectedOutput] = useState<string>("Default");

  const [expanderConfig, setExpanderConfig] = useState<ExpanderConfig>(() => {
    const defaults = {
      enabled: true,
      threshold: 0.08,
      ratio: 2.0,
      attack_ms: 10.0,
      release_ms: 100.0,
    };

    const saved = localStorage.getItem("expander_config");
    if (saved) {
      try {
        return { ...defaults, ...JSON.parse(saved) };
      } catch (e) {
        console.error("Failed to parse saved config", e);
      }
    }
    return defaults;
  });

  const [rnnoiseConfig, setRnnoiseConfig] = useState<RNNoiseConfig>(() => {
    const defaults = {
      enabled: false,
    };

    const saved = localStorage.getItem("rnnoise_config");
    if (saved) {
      try {
        return { ...defaults, ...JSON.parse(saved) };
      } catch (e) {
        console.error("Failed to parse saved config", e);
      }
    }
    return defaults;
  });

  useEffect(() => {
    localStorage.setItem("expander_config", JSON.stringify(expanderConfig));
    invoke("update_config", {
      config: {
        type: "Expander",
        data: expanderConfig,
      },
    });
  }, [expanderConfig]);

  useEffect(() => {
    localStorage.setItem("rnnoise_config", JSON.stringify(rnnoiseConfig));
    invoke("update_config", {
      config: {
        type: "RNNoise",
        data: rnnoiseConfig,
      },
    });
  }, [rnnoiseConfig]);

  const toggleEngine = () => {
    if (isRunning) {
      stopEngine();
    } else {
      startEngine(selectedInput, selectedOutput);
    }
  };

  return (
    <div className="app-container">
      <aside className="sidebar">
        <header>
          <h1>
            LEYLINE<span>FILTER</span>
          </h1>
        </header>

        <EngineControls
          isRunning={isRunning}
          inputDevices={inputDevices}
          outputDevices={outputDevices}
          selectedInput={selectedInput}
          selectedOutput={selectedOutput}
          onInputChange={setSelectedInput}
          onOutputChange={setSelectedOutput}
          onToggle={toggleEngine}
        />

        <div className="metrics-panel">
          <div className="metric">
            <label>Latency</label>
            <span>
              {metrics.latency_ms < 0
                ? "N/A"
                : `${Math.round(metrics.latency_ms)} ms`}
            </span>
          </div>
          <div className="metric">
            <label>CPU Usage</label>
            <span>{metrics.cpu_usage}%</span>
          </div>
        </div>

        <Visualizer
          isRunning={isRunning}
          spectrum={metrics.spectrum}
          tonality={metrics.tonality}
        />
      </aside>

      <main className="module-grid">
        <RNNoiseModule config={rnnoiseConfig} onChange={setRnnoiseConfig} />
        <ExpanderModule config={expanderConfig} onChange={setExpanderConfig} />

        {/* Placeholder for future modules */}
        <div className="module-card placeholder">
          <div className="module-header">
            <h3>Add Module</h3>
          </div>
          <div className="module-content">
            <button className="add-btn">+</button>
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;

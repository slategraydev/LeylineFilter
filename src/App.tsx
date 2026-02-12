// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useEngine } from "./hooks/useEngine";
import { ExpanderModule } from "./components/Modules/ExpanderModule";
import { RNNoiseModule } from "./components/Modules/RNNoiseModule";
import { GainModule } from "./components/Modules/GainModule";
import { FilterModule } from "./components/Modules/FilterModule";
import { EngineControls } from "./components/Engine/EngineControls";
import { Visualizer } from "./components/Visualizer/Visualizer";
import {
  ExpanderConfig,
  RNNoiseConfig,
  GainConfig,
  FilterConfig,
} from "./types";
import "./App.css";

/**
 * # Main Application Component
 * This component serves as the root state container for the UI.
 * It manages the local configuration state (persistence via localStorage)
 * and synchronizes with the backend via the useEngine hook.
 */
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

  /**
   * ## Local State & Persistence
   * Configuration is stored locally in React state for immediate UI responsiveness
   * and synced to the backend via useEffect hooks.
   */
  const [gainConfig, setGainConfig] = useState<GainConfig>(() => {
    const defaults = { enabled: true, gain_db: 0.0 };
    const saved = localStorage.getItem("gain_config");
    if (saved) {
      try {
        return { ...defaults, ...JSON.parse(saved) };
      } catch (e) {
        console.error(e);
      }
    }
    return defaults;
  });

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

  const [filterConfig, setFilterConfig] = useState<FilterConfig>(() => {
    const defaults: FilterConfig = {
      enabled: false,
      filter_type: "LPF",
      frequency: 1000,
      q: 0.707,
    };

    const saved = localStorage.getItem("filter_config");
    if (saved) {
      try {
        return { ...defaults, ...JSON.parse(saved) };
      } catch (e) {
        console.error("Failed to parse saved config", e);
      }
    }
    return defaults;
  });

  /**
   * ## Config Synchronization
   * We sync to the backend whenever config changes.
   * Note: The backend handles parameter smoothing, so rapid updates are safe.
   */
  useEffect(() => {
    localStorage.setItem("gain_config", JSON.stringify(gainConfig));
    invoke("update_config", {
      config: {
        type: "Gain",
        data: gainConfig,
      },
    });
  }, [gainConfig]);

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

  useEffect(() => {
    localStorage.setItem("filter_config", JSON.stringify(filterConfig));
    invoke("update_config", {
      config: {
        type: "Filter",
        data: filterConfig,
      },
    });
  }, [filterConfig]);

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
            <span data-testid="latency-value">
              {`${Math.round(metrics.latency_ms)} ms`}
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
        <GainModule config={gainConfig} onChange={setGainConfig} />
        <RNNoiseModule config={rnnoiseConfig} onChange={setRnnoiseConfig} />
        <ExpanderModule config={expanderConfig} onChange={setExpanderConfig} />
        <FilterModule config={filterConfig} onChange={setFilterConfig} />

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

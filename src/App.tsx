// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useEngine } from "./hooks/useEngine";
import { ExpanderModule } from "./components/Modules/ExpanderModule";
import { RNNoiseModule } from "./components/Modules/RNNoiseModule";
import { GainModule } from "./components/Modules/GainModule";
import { FilterModule } from "./components/Modules/FilterModule";
import { CompressorModule } from "./components/Modules/CompressorModule";
import { DeesserModule } from "./components/Modules/DeesserModule";
import { SaturationModule } from "./components/Modules/SaturationModule";
import { LimiterModule } from "./components/Modules/LimiterModule";
import { DereverbModule } from "./components/Modules/DereverbModule";
import { ParametricEQModule } from "./components/Modules/ParametricEQModule";
import { AddModuleMenu } from "./components/Engine/AddModuleMenu";
import { EngineControls } from "./components/Engine/EngineControls";
import { Oscilloscope } from "./components/Sidebar/Oscilloscope";
import { Visualizer } from "./components/Visualizer/Visualizer";
import {
  ExpanderConfig,
  RNNoiseConfig,
  GainConfig,
  FilterConfig,
  CompressorConfig,
  ParametricEQConfig,
  DeesserConfig,
  SaturationConfig,
  LimiterConfig,
  DereverbConfig,
  ModuleConfig,
} from "./types";
import { GridPosition } from "./hooks/useDraggable";
import { findFreeSlot } from "./utils/layout";
import {
  GRID_UNIT_PX,
  SIDEBAR_WIDTH_PX,
  MODULE_W_UNITS,
  MODULE_H_UNITS,
  MODULE_HEIGHTS,
} from "./constants";
import "./App.css";

/**
 * # Main Application Component
 */
function App() {
  const {
    isRunning,
    isMonitoring,
    inputDevices,
    outputDevices,
    engineState,
    metrics,
    startEngine,
    stopEngine,
    setMonitoring,
    addModule,
    removeModule,
  } = useEngine();

  const [selectedInput, setSelectedInput] = useState<string>("Default");
  const [selectedOutput, setSelectedOutput] = useState<string>("Default");
  const [showAddMenu, setShowAddMenu] = useState(false);
  const [newlyPlacedIds, setNewlyPlacedIds] = useState<Set<string>>(new Set());
  const [expectingNewModule, setExpectingNewModule] = useState(false);
  const [newlyAddedModuleIds, setNewlyAddedModuleIds] = useState<string[]>([]);
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const [draggingPos, setDraggingPos] = useState<GridPosition | null>(null);

  const [positions, setPositions] = useState<Record<string, GridPosition>>(
    () => {
      const saved = localStorage.getItem("module_positions_grid");
      if (saved) {
        try {
          return JSON.parse(saved);
        } catch (e) {
          console.error("Failed to parse module positions", e);
        }
      }
      return {};
    },
  );

  const [moduleHeights, setModuleHeights] = useState<Record<string, number>>(
    () => {
      const saved = localStorage.getItem("module_heights_grid");
      if (saved) {
        try {
          return JSON.parse(saved);
        } catch (e) {
          console.error("Failed to parse module heights", e);
        }
      }
      return {};
    },
  );

  const [moduleWidths, setModuleWidths] = useState<Record<string, number>>(
    () => {
      const saved = localStorage.getItem("module_widths_grid");
      if (saved) {
        try {
          return JSON.parse(saved);
        } catch (e) {
          console.error("Failed to parse module widths", e);
        }
      }
      return {};
    },
  );

  // Helper to find the next empty slot in the grid (used only for NEW modules)
  const findNextAvailableSlot = (
    currentPositions: Record<string, GridPosition>,
    type: string,
  ): GridPosition => {
    const availableWidthPx = window.innerWidth - SIDEBAR_WIDTH_PX - 20;
    const maxGx = Math.max(
      MODULE_W_UNITS,
      Math.floor(availableWidthPx / GRID_UNIT_PX),
    );

    const typeMap: Record<string, string> = {};
    engineState?.modules?.forEach((m) => {
      typeMap[m.id] = m.config.type;
    });
    const getH = (mid: string) =>
      moduleHeights[mid] ||
      MODULE_HEIGHTS[typeMap[mid] || "default"] ||
      MODULE_H_UNITS;
    const getW = (mid: string) => moduleWidths[mid] || MODULE_W_UNITS;

    return findFreeSlot(
      MODULE_HEIGHTS[type] || MODULE_H_UNITS,
      currentPositions,
      getH,
      getW,
      maxGx,
    );
  };

  // Sync positions when new modules appear
  useEffect(() => {
    if (!engineState) return;

    setPositions((prev) => {
      let changed = false;
      let addedIds: string[] = [];
      const next = { ...prev };

      const currentIds = new Set((engineState.modules || []).map((m) => m.id));
      Object.keys(next).forEach((id) => {
        if (!currentIds.has(id)) {
          delete next[id];
          changed = true;
        }
      });

      engineState.modules?.forEach((m) => {
        if (!next[m.id]) {
          const nextSlot = findNextAvailableSlot(next, m.config.type);
          next[m.id] = nextSlot;
          addedIds.push(m.id);
          changed = true;
        }
      });

      if (changed) {
        localStorage.setItem("module_positions_grid", JSON.stringify(next));
        if (addedIds.length > 0) {
          setNewlyAddedModuleIds((prevIds) => [...prevIds, ...addedIds]);
        }
        return next;
      }
      return prev;
    });
  }, [engineState?.modules]);

  // Handle flash animation side-effects
  useEffect(() => {
    if (newlyAddedModuleIds.length > 0) {
      if (expectingNewModule) {
        setNewlyPlacedIds((prev) => {
          const nextSet = new Set(prev);
          newlyAddedModuleIds.forEach((id) => nextSet.add(id));
          return nextSet;
        });

        // Set individual timeouts for each added module
        newlyAddedModuleIds.forEach((id) => {
          setTimeout(() => {
            setNewlyPlacedIds((prev) => {
              const nextSet = new Set(prev);
              nextSet.delete(id);
              return nextSet;
            });
          }, 1000);
        });
      }
      setExpectingNewModule(false);
      setNewlyAddedModuleIds([]);
    }
  }, [newlyAddedModuleIds, expectingNewModule]);

  const handleDrag = (
    _id: string,
    pos: GridPosition | null,
    _rawOffset?: { x: number; y: number },
    continuousPos?: GridPosition,
  ) => {
    if (!pos) {
      setDraggingId(null);
      setDraggingPos(null);
      return;
    }
    setDraggingId(_id);
    const currentDraggingPos = continuousPos || pos;
    setDraggingPos(currentDraggingPos);
  };

  const handlePositionChange = (id: string, pos: GridPosition) => {
    setDraggingId(null);
    setDraggingPos(null);

    setPositions((prev) => {
      const next = { ...prev, [id]: pos };
      localStorage.setItem("module_positions_grid", JSON.stringify(next));
      return next;
    });
  };

  // Handle global clicks to close menu
  useEffect(() => {
    if (!showAddMenu) return;

    const handleContext = (_e: MouseEvent) => {
      setShowAddMenu(false);
    };

    const handleClickOutside = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      // Close if click is outside the menu and not on the toggle button
      if (!target.closest(".add-module-menu") && !target.closest(".add-btn")) {
        setShowAddMenu(false);
      }
    };

    window.addEventListener("contextmenu", handleContext);
    window.addEventListener("mousedown", handleClickOutside);

    return () => {
      window.removeEventListener("contextmenu", handleContext);
      window.removeEventListener("mousedown", handleClickOutside);
    };
  }, [showAddMenu]);

  const addModuleFromMenu = async (moduleType: string) => {
    setExpectingNewModule(true);
    addModule(moduleType);
  };

  const removeModuleFromGrid = async (id: string) => {
    removeModule(id);
  };

  const handleInputChange = (device: string) => {
    setSelectedInput(device);
    if (isRunning) {
      startEngine(device, selectedOutput);
    }
  };

  const handleOutputChange = (device: string) => {
    setSelectedOutput(device);
    if (isRunning) {
      startEngine(selectedInput, device);
    }
  };

  const toggleEngine = () => {
    if (isRunning) {
      stopEngine();
    } else {
      startEngine(selectedInput, selectedOutput);
    }
  };

  const handleUpdateConfig = (_id: string, config: ModuleConfig) => {
    invoke("update_config", { config });
  };

  const handleHeightReport = (id: string, units: number) => {
    setModuleHeights((prev) => {
      if (prev[id] === units) return prev;
      const nextHeights = { ...prev, [id]: units };
      localStorage.setItem("module_heights_grid", JSON.stringify(nextHeights));
      return nextHeights;
    });
  };

  const getGridContentStyle = () => {
    let maxGx = 0;
    let maxGy = 0;
    const typeMap: Record<string, string> = {};
    engineState?.modules?.forEach((m) => {
      typeMap[m.id] = m.config.type;
    });

    Object.entries(positions).forEach(([id, pos]) => {
      const effectivePos = id === draggingId && draggingPos ? draggingPos : pos;
      const mType = typeMap[id] || "default";
      const mHeight =
        moduleHeights[id] || MODULE_HEIGHTS[mType] || MODULE_H_UNITS;
      const mWidth = moduleWidths[id] || MODULE_W_UNITS;
      maxGx = Math.max(maxGx, effectivePos.gx + mWidth);
      maxGy = Math.max(maxGy, effectivePos.gy + mHeight);
    });

    const effectiveGridUnit = GRID_UNIT_PX || 18;

    return {
      minWidth: "100%",
      minHeight: "100vh",
      width: maxGx > 0 ? `calc(var(--grid-unit) * ${maxGx + 5})` : "100%",
      height: maxGy > 0 ? `calc(var(--grid-unit) * ${maxGy + 10})` : "100%",
      position: "relative" as const,
      "--grid-unit": `${effectiveGridUnit}px`,
      transition: "width 0.2s ease, height 0.2s ease",
    } as React.CSSProperties;
  };

  const renderModule = (module: any) => {
    const pos = positions[module.id];
    if (!pos) return null;

    const config = module.config;
    if (!config || !config.data) return null;

    const mHeight =
      moduleHeights[module.id] || MODULE_HEIGHTS[config.type] || MODULE_H_UNITS;
    const mWidth = moduleWidths[module.id] || MODULE_W_UNITS;

    const commonProps = {
      id: module.id,
      initialPosition: pos,
      heightUnits: mHeight,
      widthUnits: mWidth,
      onPositionChange: handlePositionChange,
      onDrag: handleDrag,
      onHeightReport: handleHeightReport,
      onWidthReport: (id: string, units: number) => {
        setModuleWidths((prev) => {
          if (prev[id] === units) return prev;
          const next = { ...prev, [id]: units };
          localStorage.setItem("module_widths_grid", JSON.stringify(next));
          return next;
        });
      },
      onRemove: () => removeModuleFromGrid(module.id),
      isNewlyPlaced: newlyPlacedIds.has(module.id),
    };

    let moduleComponent = null;

    switch (config.type) {
      case "Gain":
        moduleComponent = (
          <GainModule
            key={module.id}
            {...commonProps}
            config={config.data as GainConfig}
            onChange={(data) =>
              handleUpdateConfig(module.id, { type: "Gain", data })
            }
          />
        );
        break;
      case "Expander":
        moduleComponent = (
          <ExpanderModule
            key={module.id}
            {...commonProps}
            config={config.data as ExpanderConfig}
            onChange={(data) =>
              handleUpdateConfig(module.id, { type: "Expander", data })
            }
          />
        );
        break;
      case "RNNoise":
        moduleComponent = (
          <RNNoiseModule
            key={module.id}
            {...commonProps}
            config={config.data as RNNoiseConfig}
            onChange={(data) =>
              handleUpdateConfig(module.id, { type: "RNNoise", data })
            }
          />
        );
        break;
      case "Compressor":
        moduleComponent = (
          <CompressorModule
            {...commonProps}
            config={module.config.data as CompressorConfig}
            onChange={(data) =>
              handleUpdateConfig(module.id, { type: "Compressor", data })
            }
          />
        );
        break;
      case "ParametricEQ":
        moduleComponent = (
          <ParametricEQModule
            {...commonProps}
            config={module.config.data as ParametricEQConfig}
            onChange={(data) =>
              handleUpdateConfig(module.id, { type: "ParametricEQ", data })
            }
          />
        );
        break;
      case "Deesser":
        moduleComponent = (
          <DeesserModule
            key={module.id}
            {...commonProps}
            config={config.data as DeesserConfig}
            onChange={(data) =>
              handleUpdateConfig(module.id, { type: "Deesser", data })
            }
          />
        );
        break;
      case "Saturation":
        moduleComponent = (
          <SaturationModule
            key={module.id}
            {...commonProps}
            config={config.data as SaturationConfig}
            onChange={(data) =>
              handleUpdateConfig(module.id, { type: "Saturation", data })
            }
          />
        );
        break;
      case "Limiter":
        moduleComponent = (
          <LimiterModule
            key={module.id}
            {...commonProps}
            config={config.data as LimiterConfig}
            onUpdate={(data) =>
              handleUpdateConfig(module.id, { type: "Limiter", data })
            }
            onRemove={() => removeModule(module.id)}
            onToggle={(enabled) =>
              handleUpdateConfig(module.id, {
                type: "Limiter",
                data: { ...(config.data as LimiterConfig), enabled },
              })
            }
          />
        );
        break;
      case "Dereverb":
        moduleComponent = (
          <DereverbModule
            key={module.id}
            {...commonProps}
            config={config.data as DereverbConfig}
            onUpdate={(data) =>
              handleUpdateConfig(module.id, { type: "Dereverb", data })
            }
            onRemove={() => removeModule(module.id)}
            onToggle={(enabled) =>
              handleUpdateConfig(module.id, {
                type: "Dereverb",
                data: { ...(config.data as DereverbConfig), enabled },
              })
            }
          />
        );
        break;
      case "Filter":
        moduleComponent = (
          <FilterModule
            key={module.id}
            {...commonProps}
            config={config.data as FilterConfig}
            onChange={(data) =>
              handleUpdateConfig(module.id, { type: "Filter", data })
            }
          />
        );
        break;
      case "Visualizer":
        // Removed from grid: rendered in sidebar now.
        return null;
    }

    return moduleComponent;
  };

  return (
    <div className="app-container">
      <aside className="sidebar">
        <header>
          <h1>
            LEYLINE<span>FILTER</span>
          </h1>
          <p className="sidebar-description">Neural Audio Engine</p>
        </header>

        <div className="sidebar-content">
          <div className="sidebar-section">
            <EngineControls
              isRunning={isRunning}
              isMonitoring={isMonitoring}
              inputDevices={inputDevices}
              outputDevices={outputDevices}
              selectedInput={selectedInput}
              selectedOutput={selectedOutput}
              onInputChange={handleInputChange}
              onOutputChange={handleOutputChange}
              onToggleMonitoring={setMonitoring}
            />
          </div>

          <div className="sidebar-section">
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
              <div className="metric">
                <label>Sample Rate</label>
                <span>
                  {isRunning && engineState
                    ? `${(engineState.sample_rate / 1000).toFixed(1)} kHz`
                    : "---"}
                </span>
              </div>
              <div className="metric">
                <label>Buffer</label>
                <span>
                  {isRunning
                    ? `${metrics.buffer_size || (engineState?.buffer_size ?? "---")} smp`
                    : "---"}
                </span>
              </div>
            </div>
          </div>
          <div className="sidebar-section visualizer-top">
            <Oscilloscope waveform={metrics.waveform} isRunning={isRunning} />
            <Visualizer
              isRunning={isRunning}
              spectrum={metrics.spectrum}
              tonality={metrics.tonality}
            />
          </div>
        </div>

        <div className="sidebar-footer">
          <button
            className={`engine-toggle ${isRunning ? "stop" : "start"}`}
            onClick={toggleEngine}
          >
            {isRunning ? "Stop Engine" : "Start Engine"}
          </button>
        </div>
      </aside>

      <main className="module-grid">
        <div className="grid-inner" style={getGridContentStyle()}>
          {engineState?.modules?.map((m) => {
            try {
              return renderModule(m);
            } catch (e) {
              console.error(`Failed to render module ${m.id}:`, e);
              return null;
            }
          })}
        </div>
      </main>

      {showAddMenu && (
        <AddModuleMenu
          onAdd={addModuleFromMenu}
          onClose={() => setShowAddMenu(false)}
          existingTypes={(engineState?.modules || []).map((m) => m.config.type)}
        />
      )}

      <div className="module-card placeholder">
        <button
          className="add-btn"
          aria-label="Add Module"
          onClick={() => setShowAddMenu(!showAddMenu)}
        >
          <svg viewBox="0 0 24 24" className="add-icon">
            <line x1="12" y1="5" x2="12" y2="19" />
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
        </button>
      </div>
    </div>
  );
}

export default App;

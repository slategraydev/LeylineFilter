// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useEngine } from "./hooks/useEngine";
import { ExpanderModule } from "./components/Modules/ExpanderModule";
import { RNNoiseModule } from "./components/Modules/RNNoiseModule";
import { GainModule } from "./components/Modules/GainModule";
import { FilterModule } from "./components/Modules/FilterModule";
import { VisualizerModule } from "./components/Modules/VisualizerModule";
import { AddModuleMenu } from "./components/Engine/AddModuleMenu";
import { EngineControls } from "./components/Engine/EngineControls";
import {
  ExpanderConfig,
  RNNoiseConfig,
  GainConfig,
  FilterConfig,
  ModuleConfig,
} from "./types";
import { GridPosition } from "./hooks/useDraggable";
import {
  GRID_UNIT_PX,
  SIDEBAR_WIDTH_PX,
  MODULE_W_UNITS,
  MODULE_H_UNITS,
  MODULE_HEIGHTS,
  GAP_UNITS
} from "./constants";
import "./App.css";

/**
 * # Main Application Component
 */
function App() {
  const {
    isRunning,
    inputDevices,
    outputDevices,
    engineState,
    metrics,
    startEngine,
    stopEngine,
    addModule,
    removeModule,
  } = useEngine();

  const [selectedInput, setSelectedInput] = useState<string>("Default");
  const [selectedOutput, setSelectedOutput] = useState<string>("Default");
  const [showAddMenu, setShowAddMenu] = useState(false);

  const [positions, setPositions] = useState<Record<string, GridPosition>>(() => {
    const saved = localStorage.getItem("module_positions_grid");
    if (saved) {
      try {
        return JSON.parse(saved);
      } catch (e) {
        console.error("Failed to parse module positions", e);
      }
    }
    return {};
  });

  // Helper to find the next empty slot in the grid (using 1x1 units)
  const findNextAvailableSlot = (currentPositions: Record<string, GridPosition>, type: string): GridPosition => {
    // 20px buffer for scrollbars and right-side breathing room
    const availableWidthPx = window.innerWidth - SIDEBAR_WIDTH_PX - 20;
    const availableUnits = Math.floor(availableWidthPx / GRID_UNIT_PX);

    // Safety check: always allow at least one module width
    const maxGx = Math.max(MODULE_W_UNITS, availableUnits);
    const existingIds = Object.keys(currentPositions);

    // We need the engineState to look up types of existing modules
    const moduleTypeMap: Record<string, string> = {};
    if (engineState) {
      engineState.modules.forEach(m => {
        moduleTypeMap[m.id] = m.config.type;
      });
    }

    const h = MODULE_HEIGHTS[type] || MODULE_H_UNITS;
    // If no modules, start at (1,1)
    if (existingIds.length === 0) return { gx: 1, gy: 1 };

    // Intelligent "Anchor Point" Search:
    // We check (1,1) and points adjacent to the right/bottom of existing modules.
    const potentialYs = [1];
    const potentialXs = [1];

    existingIds.forEach(id => {
      const pos = currentPositions[id];
      const mType = moduleTypeMap[id] || "default";
      const mHeight = MODULE_HEIGHTS[mType] || MODULE_H_UNITS;

      potentialYs.push(pos.gy + mHeight + GAP_UNITS);
      potentialXs.push(pos.gx + MODULE_W_UNITS + GAP_UNITS);
    });

    const uniqueYs = Array.from(new Set(potentialYs)).sort((a, b) => a - b);
    const uniqueXs = Array.from(new Set(potentialXs)).sort((a, b) => a - b);

    for (const gy of uniqueYs) {
      for (const gx of uniqueXs) {
        // Horizontal clipping check
        if (gx + MODULE_W_UNITS - 1 > maxGx) continue;

        // Check if this slot is blocked by any existing module
        const isAreaBlocked = existingIds.some(id => {
          const pos = currentPositions[id];
          const mType = moduleTypeMap[id] || "default";
          const mHeight = MODULE_HEIGHTS[mType] || MODULE_H_UNITS;

          const hOverlap = gx < pos.gx + MODULE_W_UNITS + GAP_UNITS && gx + MODULE_W_UNITS + GAP_UNITS > pos.gx;
          const vOverlap = gy < pos.gy + mHeight + GAP_UNITS && gy + h + GAP_UNITS > pos.gy;
          return hOverlap && vOverlap;
        });

        if (!isAreaBlocked) {
          return { gx, gy };
        }
      }
    }

    return { gx: 1, gy: 1 };
  };

  // Sync positions when new modules appear
  useEffect(() => {
    if (!engineState) return;

    setPositions(prev => {
      let changed = false;
      const next = { ...prev };

      // Prune stale positions for modules that no longer exist in the engine
      const currentIds = new Set((engineState.modules || []).map(m => m.id));
      Object.keys(next).forEach(id => {
        if (!currentIds.has(id)) {
          delete next[id];
          changed = true;
        }
      });

      // Add positions for new modules
      engineState.modules?.forEach(m => {
        if (!next[m.id]) {
          const nextSlot = findNextAvailableSlot(next, m.config.type);
          next[m.id] = nextSlot;
          changed = true;
        }
      });

      if (changed) {
        localStorage.setItem("module_positions_grid", JSON.stringify(next));
        return next;
      }
      return prev;
    });
  }, [engineState?.modules]);

  const handlePositionChange = (id: string, pos: GridPosition) => {
    setPositions((prev) => {
      // Helper to check if a specific position is blocked for a given module
      const isBlocked = (targetPos: GridPosition, targetId: string, currentPos: Record<string, GridPosition>) => {
        const moduleTypeMap: Record<string, string> = {};
        engineState?.modules?.forEach(m => {
          moduleTypeMap[m.id] = m.config.type;
        });

        const myType = moduleTypeMap[targetId] || "default";
        const myHeight = MODULE_HEIGHTS[myType] || MODULE_H_UNITS;

        return Object.entries(currentPos).some(([oid, otherPos]) => {
          if (oid === targetId) return false;
          const otherType = moduleTypeMap[oid] || "default";
          const otherHeight = MODULE_HEIGHTS[otherType] || MODULE_H_UNITS;

          const hOverlap = targetPos.gx < otherPos.gx + MODULE_W_UNITS + GAP_UNITS && targetPos.gx + MODULE_W_UNITS + GAP_UNITS > otherPos.gx;
          const vOverlap = targetPos.gy < otherPos.gy + otherHeight + GAP_UNITS && targetPos.gy + myHeight + GAP_UNITS > otherPos.gy;
          return hOverlap && vOverlap;
        });
      };

      if (!isBlocked(pos, id, prev)) {
        const next = { ...prev, [id]: pos };
        localStorage.setItem("module_positions_grid", JSON.stringify(next));
        return next;
      }

      // If blocked, find the nearest available anchor point to the dropped position
      const availableWidthPx = window.innerWidth - SIDEBAR_WIDTH_PX - 20;
      const availableUnits = Math.floor(availableWidthPx / GRID_UNIT_PX);
      const maxGx = Math.max(MODULE_W_UNITS, availableUnits);

      const moduleTypeMap: Record<string, string> = {};
      engineState?.modules?.forEach(m => {
        moduleTypeMap[m.id] = m.config.type;
      });

      const potentialYs = [1, pos.gy];
      const potentialXs = [1, pos.gx];

      Object.entries(prev).forEach(([oid, otherPos]) => {
        if (oid === id) return;
        const otherType = moduleTypeMap[oid] || "default";
        const otherHeight = MODULE_HEIGHTS[otherType] || MODULE_H_UNITS;
        potentialYs.push(otherPos.gy + otherHeight + GAP_UNITS);
        potentialXs.push(otherPos.gx + MODULE_W_UNITS + GAP_UNITS);
      });

      const uniqueYs = Array.from(new Set(potentialYs));
      const uniqueXs = Array.from(new Set(potentialXs));

      let bestPos = prev[id]; // Fallback to old position
      let minDistance = Infinity;

      for (const gy of uniqueYs) {
        for (const gx of uniqueXs) {
          if (gx + MODULE_W_UNITS - 1 > maxGx) continue;
          if (gx < 1 || gy < 1) continue;

          if (!isBlocked({ gx, gy }, id, prev)) {
            const dist = Math.sqrt(Math.pow(gx - pos.gx, 2) + Math.pow(gy - pos.gy, 2));
            if (dist < minDistance) {
              minDistance = dist;
              bestPos = { gx, gy };
            }
          }
        }
      }

      const next = { ...prev, [id]: bestPos };
      localStorage.setItem("module_positions_grid", JSON.stringify(next));
      return next;
    });
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

  // Calculate container dimensions based on module positions
  const getGridContentStyle = () => {
    let maxGx = 0;
    let maxGy = 0;

    // Create a temporary map of types from engineState if available
    const typeMap: Record<string, string> = {};
    engineState?.modules?.forEach(m => {
      typeMap[m.id] = m.config.type;
    });

    Object.entries(positions).forEach(([id, pos]) => {
      const mType = typeMap[id] || "default";
      const mHeight = MODULE_HEIGHTS[mType] || MODULE_H_UNITS;

      maxGx = Math.max(maxGx, pos.gx + MODULE_W_UNITS);
      maxGy = Math.max(maxGy, pos.gy + mHeight);
    });

    return {
      minWidth: `${maxGx * GRID_UNIT_PX + 40}px`,
      minHeight: `${maxGy * GRID_UNIT_PX + 100}px`,
      position: 'relative' as const,
    };
  };

  const renderModule = (module: any) => {
    // FIX: To prevent the "flash" artifact at (1,1), we check if the position exists.
    // If not, we don't render it for ONE frame until the useEffect calculates the slot.
    // This makes the module "pop" into the correct spot immediately.
    const pos = positions[module.id];
    if (!pos) return null;

    const config = module.config;
    const mHeight = MODULE_HEIGHTS[config.type] || MODULE_H_UNITS;

    const commonProps = {
      id: module.id,
      initialPosition: pos,
      heightUnits: mHeight,
      onPositionChange: handlePositionChange,
      onRemove: () => removeModule(module.id),
    };

    let moduleComponent = null;

    switch (config.type) {
      case "Gain":
        moduleComponent = (
          <GainModule
            key={module.id}
            {...commonProps}
            config={config.data as GainConfig}
            onChange={(data) => handleUpdateConfig(module.id, { type: "Gain", data })}
          />
        );
        break;
      case "Expander":
        moduleComponent = (
          <ExpanderModule
            key={module.id}
            {...commonProps}
            config={config.data as ExpanderConfig}
            onChange={(data) => handleUpdateConfig(module.id, { type: "Expander", data })}
          />
        );
        break;
      case "RNNoise":
        moduleComponent = (
          <RNNoiseModule
            key={module.id}
            {...commonProps}
            config={config.data as RNNoiseConfig}
            onChange={(data) => handleUpdateConfig(module.id, { type: "RNNoise", data })}
          />
        );
        break;
      case "Filter":
        moduleComponent = (
          <FilterModule
            key={module.id}
            {...commonProps}
            config={config.data as FilterConfig}
            onChange={(data) => handleUpdateConfig(module.id, { type: "Filter", data })}
          />
        );
        break;
      case "Visualizer":
        moduleComponent = (
          <VisualizerModule
            key={module.id}
            {...commonProps}
            enabled={config.data.enabled}
            onToggle={(enabled) => handleUpdateConfig(module.id, { type: "Visualizer", data: { enabled } })}
            isRunning={isRunning}
            spectrum={metrics.spectrum}
            tonality={metrics.tonality}
          />
        );
        break;
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

        <div className="sidebar-section">
          <div className="section-label">
            <span>I/O Config</span>
          </div>
          <EngineControls
            isRunning={isRunning}
            inputDevices={inputDevices}
            outputDevices={outputDevices}
            selectedInput={selectedInput}
            selectedOutput={selectedOutput}
            onInputChange={setSelectedInput}
            onOutputChange={setSelectedOutput}
          />
        </div>

        <div className="sidebar-section">
          <div className="section-label">
            <span>System Telemetry</span>
          </div>
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
              <span>{isRunning && engineState ? `${(engineState.sample_rate / 1000).toFixed(1)} kHz` : "---"}</span>
            </div>
            <div className="metric">
              <label>Buffer</label>
              <span>{isRunning ? `${metrics.buffer_size || (engineState?.buffer_size ?? "---")} smp` : "---"}</span>
            </div>
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

      <main className="module-grid" style={getGridContentStyle()}>
        {engineState?.modules?.map(renderModule)}

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

        {showAddMenu && (
          <AddModuleMenu
            onAdd={(type) => {
              addModule(type);
              setShowAddMenu(false);
            }}
            onClose={() => setShowAddMenu(false)}
          />
        )}
      </main>
    </div>
  );
}

export default App;

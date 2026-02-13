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
import { findFreeSlot, calculateScale } from "./utils/layout";
import {
  GRID_UNIT_PX,
  SIDEBAR_WIDTH_PX,
  MODULE_W_UNITS,
  MODULE_H_UNITS,
  MODULE_HEIGHTS
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
  const [newlyPlacedId, setNewlyPlacedId] = useState<string | null>(null);
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const [draggingPos, setDraggingPos] = useState<GridPosition | null>(null);

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

  const [scale, setScale] = useState(1.0);

  const [moduleHeights, setModuleHeights] = useState<Record<string, number>>(() => {
    const saved = localStorage.getItem("module_heights_grid");
    if (saved) {
      try {
        return JSON.parse(saved);
      } catch (e) {
        console.error("Failed to parse module heights", e);
      }
    }
    return {};
  });

  // Helper to find the next empty slot in the grid (used only for NEW modules)
  const findNextAvailableSlot = (currentPositions: Record<string, GridPosition>, type: string): GridPosition => {
    const availableWidthPx = window.innerWidth - SIDEBAR_WIDTH_PX - 20;
    const maxGx = Math.max(MODULE_W_UNITS, Math.floor(availableWidthPx / GRID_UNIT_PX));

    const typeMap: Record<string, string> = {};
    engineState?.modules?.forEach(m => { typeMap[m.id] = m.config.type; });
    const getH = (mid: string) => moduleHeights[mid] || MODULE_HEIGHTS[typeMap[mid] || "default"] || MODULE_H_UNITS;

    return findFreeSlot(MODULE_HEIGHTS[type] || MODULE_H_UNITS, currentPositions, getH, maxGx);
  };

  // Sync scale when window is resized
  useEffect(() => {
    const handleResize = () => {
      setPositions(prev => {
        if (Object.keys(prev).length === 0) return prev;

        const typeMap: Record<string, string> = {};
        engineState?.modules?.forEach(m => { typeMap[m.id] = m.config.type; });
        const getH = (mid: string) => moduleHeights[mid] || MODULE_HEIGHTS[typeMap[mid] || "default"] || MODULE_H_UNITS;

        const availableWidthPx = window.innerWidth - SIDEBAR_WIDTH_PX - 20;
        const availableHeightPx = window.innerHeight - 20;
        const maxGx = Math.max(MODULE_W_UNITS, Math.floor(availableWidthPx / GRID_UNIT_PX));
        const maxGy = Math.max(MODULE_H_UNITS, Math.floor(availableHeightPx / GRID_UNIT_PX));

        setScale(calculateScale(prev, getH, maxGx, maxGy));
        return prev;
      });
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, [engineState?.modules, moduleHeights]);

  // Sync positions when new modules appear
  useEffect(() => {
    if (!engineState) return;

    setPositions(prev => {
      let changed = false;
      let newIdToFlash: string | null = null;
      const next = { ...prev };

      const currentIds = new Set((engineState.modules || []).map(m => m.id));
      Object.keys(next).forEach(id => {
        if (!currentIds.has(id)) {
          delete next[id];
          changed = true;
        }
      });

      engineState.modules?.forEach(m => {
        if (!next[m.id]) {
          const nextSlot = findNextAvailableSlot(next, m.config.type);
          next[m.id] = nextSlot;
          newIdToFlash = m.id;
          changed = true;
        }
      });

      if (changed) {
        const typeMap: Record<string, string> = {};
        engineState.modules.forEach(m => { typeMap[m.id] = m.config.type; });
        const getH = (mid: string) => moduleHeights[mid] || MODULE_HEIGHTS[typeMap[mid] || "default"] || MODULE_H_UNITS;

        const availableWidthPx = window.innerWidth - SIDEBAR_WIDTH_PX - 20;
        const availableHeightPx = window.innerHeight - 20;
        const maxGx = Math.max(MODULE_W_UNITS, Math.floor(availableWidthPx / GRID_UNIT_PX));
        const maxGy = Math.max(MODULE_H_UNITS, Math.floor(availableHeightPx / GRID_UNIT_PX));

        setScale(calculateScale(next, getH, maxGx, maxGy));
        localStorage.setItem("module_positions_grid", JSON.stringify(next));

        if (newIdToFlash) {
          setNewlyPlacedId(newIdToFlash);
          setTimeout(() => setNewlyPlacedId(null), 2000);
        }

        return next;
      }
      return prev;
    });
  }, [engineState?.modules]);

  const handleDrag = (id: string, pos: GridPosition | null, _rawOffset?: { x: number, y: number }, continuousPos?: GridPosition) => {
    if (!pos) {
      setDraggingId(null);
      setDraggingPos(null);
      return;
    }
    setDraggingId(id);
    const currentDraggingPos = continuousPos || pos;
    setDraggingPos(currentDraggingPos);

    const typeMap: Record<string, string> = {};
    engineState?.modules?.forEach(m => {
      typeMap[m.id] = m.config.type;
    });
    const getH = (mid: string) => moduleHeights[mid] || MODULE_HEIGHTS[typeMap[mid] || "default"] || MODULE_H_UNITS;

    const availableWidthPx = window.innerWidth - SIDEBAR_WIDTH_PX - 20;
    const availableHeightPx = window.innerHeight - 20;
    const maxGx = Math.max(MODULE_W_UNITS, Math.floor(availableWidthPx / GRID_UNIT_PX));
    const maxGy = Math.max(MODULE_H_UNITS, Math.floor(availableHeightPx / GRID_UNIT_PX));

    // Use continuous (raw) position for scale calculation to make it smooth
    const tempPositions = { ...positions, [id]: currentDraggingPos };
    const newScale = calculateScale(tempPositions, getH, maxGx, maxGy);

    if (newScale !== scale) {
      setScale(newScale);
    }
  };

  const handlePositionChange = (id: string, pos: GridPosition) => {
    setDraggingId(null);
    setDraggingPos(null);

    setPositions(prev => {
      const typeMap: Record<string, string> = {};
      engineState?.modules?.forEach(m => {
        typeMap[m.id] = m.config.type;
      });
      const getH = (mid: string) => moduleHeights[mid] || MODULE_HEIGHTS[typeMap[mid] || "default"] || MODULE_H_UNITS;

      const availableWidthPx = window.innerWidth - SIDEBAR_WIDTH_PX - 20;
      const availableHeightPx = window.innerHeight - 20;
      const maxGx = Math.max(MODULE_W_UNITS, Math.floor(availableWidthPx / GRID_UNIT_PX));
      const maxGy = Math.max(MODULE_H_UNITS, Math.floor(availableHeightPx / GRID_UNIT_PX));

      const next = { ...prev, [id]: pos };
      setScale(calculateScale(next, getH, maxGx, maxGy));
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
      if (!target.closest('.add-module-menu') && !target.closest('.add-btn')) {
        setShowAddMenu(false);
      }
    };

    window.addEventListener('contextmenu', handleContext);
    window.addEventListener('mousedown', handleClickOutside);

    return () => {
      window.removeEventListener('contextmenu', handleContext);
      window.removeEventListener('mousedown', handleClickOutside);
    };
  }, [showAddMenu]);

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
    setModuleHeights(prev => {
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
    engineState?.modules?.forEach(m => { typeMap[m.id] = m.config.type; });

    Object.entries(positions).forEach(([id, pos]) => {
      const effectivePos = (id === draggingId && draggingPos) ? draggingPos : pos;
      const mType = typeMap[id] || "default";
      const mHeight = moduleHeights[id] || MODULE_HEIGHTS[mType] || MODULE_H_UNITS;
      maxGx = Math.max(maxGx, effectivePos.gx + MODULE_W_UNITS);
      maxGy = Math.max(maxGy, effectivePos.gy + mHeight);
    });

    return {
      minWidth: `${maxGx * GRID_UNIT_PX + 20}px`,
      minHeight: `${maxGy * GRID_UNIT_PX + 20}px`,
      position: 'relative' as const,
      transform: `scale(${scale})`,
      transformOrigin: 'top left',
      transition: draggingId ? 'none' : 'transform 0.3s cubic-bezier(0.2, 0.8, 0.2, 1)',
    };
  };

  const renderModule = (module: any) => {
    const pos = positions[module.id];
    if (!pos) return null;

    const config = module.config;
    const mHeight = moduleHeights[module.id] || MODULE_HEIGHTS[config.type] || MODULE_H_UNITS;

    const commonProps = {
      id: module.id,
      initialPosition: pos,
      heightUnits: mHeight,
      scale: scale,
      onPositionChange: handlePositionChange,
      onDrag: handleDrag,
      onHeightReport: handleHeightReport,
      onRemove: () => removeModule(module.id),
      isNewlyPlaced: newlyPlacedId === module.id,
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

      <main className="module-grid">
        <div className="grid-inner" style={getGridContentStyle()}>
          {engineState?.modules?.map(renderModule)}
        </div>
      </main>

      {showAddMenu && (
        <AddModuleMenu
          onAdd={(type) => {
            addModule(type);
            // Don't close menu here
          }}
          onClose={() => setShowAddMenu(false)}
          existingTypes={(engineState?.modules || []).map(m => m.config.type)}
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

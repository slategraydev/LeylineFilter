// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { EngineMetrics, EngineState } from "../types";

/**
 * # Engine Hook
 * The primary orchestration hook for the frontend.
 * It manages the connection to the Rust backend via Tauri commands.
 */
export function useEngine() {
  const [isRunning, setIsRunning] = useState(false);
  const [isMonitoring, setIsMonitoring] = useState(true);
  const [inputDevices, setInputDevices] = useState<string[]>([]);
  const [outputDevices, setOutputDevices] = useState<string[]>([]);
  const [engineState, setEngineState] = useState<EngineState | null>(null);
  const [metrics, setMetrics] = useState<EngineMetrics>({
    latency_ms: 0,
    cpu_usage: 0,
    input_level: 0,
    input_level_db: -60,
    buffer_size: 256,
    spectrum: Array(12).fill(0),
    tonality: Array(12).fill(0),
    waveform: Array(64).fill(0),
    state_version: 0,
  });

  const [lastVersion, setLastVersion] = useState<number>(0);

  useEffect(() => {
    // --- Initialization ---
    // This only runs once on mount to populate the device lists
    // and fetch the initial engine state.
    invoke<string[]>("get_input_devices").then((d) =>
      setInputDevices(Array.from(new Set(["Default", ...d]))),
    );
    invoke<string[]>("get_output_devices").then((d) =>
      setOutputDevices(Array.from(new Set(["Default", ...d]))),
    );

    // Initial state fetch
    invoke<EngineState>("get_engine_state").then((state) => {
      setEngineState(state);
      setIsRunning(state.is_running);
    });
  }, []);

  useEffect(() => {
    /**
     * ## High-Frequency Polling
     * We poll metrics every 33ms (~30fps) instead of using events.
     * Events would flood the IPC channel and cause backpressure on the audio thread.
     * Polling lets the UI dictate the update rate.
     */
    const interval = setInterval(async () => {
      try {
        const m = await invoke<EngineMetrics & { state_version: number }>(
          "get_metrics",
        );

        // Comprehensive safety check for every property
        const safeMetrics: EngineMetrics = {
          latency_ms: m?.latency_ms ?? 0,
          cpu_usage: m?.cpu_usage ?? 0,
          input_level: m?.input_level ?? 0,
          input_level_db: m?.input_level_db ?? -60,
          buffer_size: m?.buffer_size ?? 256,
          spectrum: Array.isArray(m?.spectrum) ? m.spectrum : Array(12).fill(0),
          tonality: Array.isArray(m?.tonality) ? m.tonality : Array(12).fill(0),
          waveform: Array.isArray(m?.waveform) ? m.waveform : Array(64).fill(0),
          state_version: m?.state_version ?? 0,
        };

        setMetrics(safeMetrics);

        /**
         * ## Efficient State Sync
         * We only fetch the full EngineState (which can be large) if the backend
         * reports a version change (e.g., a module was added or removed).
         */
        if (safeMetrics.state_version !== lastVersion) {
          const state = await invoke<EngineState>("get_engine_state");
          if (state) {
            setEngineState(state);
            setIsRunning(state.is_running ?? false);
            setIsMonitoring(state.monitoring_enabled ?? true);
            setLastVersion(safeMetrics.state_version);
          }
        }
      } catch (e) {
        // Silently fail if backend not ready (e.g. app closing)
      }
    }, 33);
    return () => clearInterval(interval);
  }, [lastVersion]); // Dependency on lastVersion to ensure state updates correctly in closure

  const setMonitoring = async (enabled: boolean) => {
    try {
      await invoke("set_monitoring", { enabled });
      setIsMonitoring(enabled);
    } catch (e) {
      console.error("Failed to toggle monitoring:", e);
    }
  };

  const startEngine = async (inputDevice: string, outputDevice: string) => {
    try {
      console.log(
        `Starting engine with Input: ${inputDevice}, Output: ${outputDevice}`,
      );
      await invoke("start_engine", {
        input_device: inputDevice === "Default" ? null : inputDevice,
        output_device: outputDevice === "Default" ? null : outputDevice,
      });
      setIsRunning(true);
      // Force a state refresh immediately after start
      const state = await invoke<EngineState>("get_engine_state");
      setEngineState(state);
      setLastVersion(0);
    } catch (e) {
      console.error("Failed to start engine:", e);
      alert(`Engine Error: ${e}`);
      setIsRunning(false);
    }
  };

  const stopEngine = async () => {
    await invoke("stop_engine");
    setIsRunning(false);
  };

  const addModule = async (moduleType: string) => {
    try {
      await invoke("send_command", {
        command: {
          type: "AddModule",
          data: { module_type: moduleType },
        },
      });
    } catch (e) {
      console.error("Failed to add module:", e);
    }
  };

  const removeModule = async (id: string) => {
    try {
      await invoke("send_command", {
        command: {
          type: "RemoveModule",
          data: { id },
        },
      });
    } catch (e) {
      console.error("Failed to remove module:", e);
    }
  };

  return {
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
  };
}

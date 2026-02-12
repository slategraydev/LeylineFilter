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
  const [inputDevices, setInputDevices] = useState<string[]>([]);
  const [outputDevices, setOutputDevices] = useState<string[]>([]);
  const [engineState, setEngineState] = useState<EngineState | null>(null);
  const [metrics, setMetrics] = useState<EngineMetrics>({
    latency_ms: 0,
    cpu_usage: 0,
    input_level: 0,
    spectrum: Array(12).fill(0),
    tonality: Array(12).fill(0),
    state_version: 0,
  });

  const [lastVersion, setLastVersion] = useState<number>(0);

  useEffect(() => {
    // --- Initialization ---
    invoke<string[]>("get_input_devices").then((d) =>
      setInputDevices(["Default", ...d]),
    );
    invoke<string[]>("get_output_devices").then((d) =>
      setOutputDevices(["Default", ...d]),
    );

    // Initial state fetch
    invoke<EngineState>("get_engine_state").then((state) => {
      setEngineState(state);
      setIsRunning(state.is_running);
    });

    /**
     * ## High-Frequency Polling
     * We poll metrics every 33ms (~30fps) instead of using events.
     * Events would flood the IPC channel and cause backpressure on the audio thread.
     * Polling lets the UI dictate the update rate.
     */
    const interval = setInterval(async () => {
      try {
        const m = await invoke<EngineMetrics & { state_version: number }>("get_metrics");
        setMetrics(m);

        /**
         * ## Efficient State Sync
         * We only fetch the full EngineState (which can be large) if the backend
         * reports a version change (e.g., a module was added or removed).
         */
        if (m.state_version !== lastVersion) {
          const state = await invoke<EngineState>("get_engine_state");
          setEngineState(state);
          setIsRunning(state.is_running);
          setLastVersion(m.state_version);
        }
      } catch (e) {
        // Silently fail if backend not ready (e.g. app closing)
      }
    }, 33);
    return () => clearInterval(interval);
  }, [lastVersion]); // Dependency on lastVersion to ensure state updates correctly in closure

  const startEngine = async (inputDevice: string, outputDevice: string) => {
    await invoke("start_engine", {
      input_device: inputDevice === "Default" ? null : inputDevice,
      output_device: outputDevice === "Default" ? null : outputDevice,
    });
    setIsRunning(true);
  };

  const stopEngine = async () => {
    await invoke("stop_engine");
    setIsRunning(false);
  };

  return {
    isRunning,
    inputDevices,
    outputDevices,
    engineState,
    metrics,
    startEngine,
    stopEngine,
  };
}

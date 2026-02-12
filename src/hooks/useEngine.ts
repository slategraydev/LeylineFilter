import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { EngineMetrics, EngineState } from "../types";

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

    const interval = setInterval(async () => {
      try {
        const m = await invoke<EngineMetrics & { state_version: number }>("get_metrics");
        setMetrics(m);

        // Sync full state only if version changed
        if (m.state_version !== lastVersion) {
          const state = await invoke<EngineState>("get_engine_state");
          setEngineState(state);
          setIsRunning(state.is_running);
          setLastVersion(m.state_version);
        }
      } catch (e) {
        // Silently fail if backend not ready
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

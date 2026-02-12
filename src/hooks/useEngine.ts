import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { EngineMetrics } from "../types";

export function useEngine() {
  const [isRunning, setIsRunning] = useState(false);
  const [inputDevices, setInputDevices] = useState<string[]>([]);
  const [outputDevices, setOutputDevices] = useState<string[]>([]);
  const [metrics, setMetrics] = useState<EngineMetrics>({
    latency_ms: 0,
    cpu_usage: 0,
    input_level: 0,
    spectrum: Array(12).fill(0),
    tonality: Array(12).fill(0),
  });

  useEffect(() => {
    invoke<string[]>("get_input_devices").then((d) =>
      setInputDevices(["Default", ...d]),
    );
    invoke<string[]>("get_output_devices").then((d) =>
      setOutputDevices(["Default", ...d]),
    );

    const interval = setInterval(async () => {
      try {
        const m = await invoke<EngineMetrics>("get_metrics");
        setMetrics(m);
      } catch (e) {
        // Silently fail if backend not ready
      }
    }, 33);
    return () => clearInterval(interval);
  }, []);

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
    metrics,
    startEngine,
    stopEngine,
  };
}

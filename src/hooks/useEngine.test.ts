import { renderHook, act, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { useEngine } from "./useEngine";
import { invoke } from "@tauri-apps/api/core";

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("useEngine", () => {
  const mockEngineState = {
    modules: [],
    is_running: false,
    sample_rate: 48000,
  };

  const mockMetrics = {
    latency_ms: 10,
    cpu_usage: 5,
    input_level: 0.1,
    spectrum: Array(12).fill(0.1),
    tonality: Array(12).fill(0.1),
    state_version: 1,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      // console.log(`[Mock Invoke] ${cmd}`); // Uncomment for debugging
      if (cmd === "get_input_devices") return ["Mic 1"];
      if (cmd === "get_output_devices") return ["Speakers 1"];
      if (cmd === "get_engine_state") return mockEngineState;
      if (cmd === "get_metrics") return mockMetrics;
      if (cmd === "start_engine") return null;
      if (cmd === "stop_engine") return null;
      // Default fallback to prevent 'undefined' crashes if a new command is added
      return [];
    });
  });

  it("initializes with default devices and state", async () => {
    const { result } = renderHook(() => useEngine());

    await waitFor(() => {
      expect(result.current.inputDevices).toEqual(["Default", "Mic 1"]);
      expect(result.current.outputDevices).toEqual(["Default", "Speakers 1"]);
      expect(result.current.engineState).toEqual(mockEngineState);
    });
  });

  it("starts the engine with correct parameters", async () => {
    const { result } = renderHook(() => useEngine());

    await act(async () => {
      await result.current.startEngine("Mic 1", "Speakers 1");
    });

    expect(invoke).toHaveBeenCalledWith("start_engine", {
      inputDevice: "Mic 1",
      outputDevice: "Speakers 1",
      monitorDevice: null,
    });
    expect(result.current.isRunning).toBe(true);
  });

  it('handles "Default" device selection correctly', async () => {
    const { result } = renderHook(() => useEngine());

    await act(async () => {
      await result.current.startEngine("Default", "Default");
    });

    expect(invoke).toHaveBeenCalledWith("start_engine", {
      inputDevice: "Default",
      outputDevice: "Default",
      monitorDevice: null,
    });
  });

  it("stops the engine", async () => {
    const { result } = renderHook(() => useEngine());

    // First start it
    await act(async () => {
      await result.current.startEngine("Mic 1", "Speakers 1");
    });

    // Then stop it
    await act(async () => {
      await result.current.stopEngine();
    });

    expect(invoke).toHaveBeenCalledWith("stop_engine");
    expect(result.current.isRunning).toBe(false);
  });

  it("updates state when version changes", async () => {
    const { result } = renderHook(() => useEngine());

    // Mock initial state
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "get_input_devices") return ["Mic 1"];
      if (cmd === "get_output_devices") return ["Speakers 1"];
      if (cmd === "get_metrics") return { ...mockMetrics, state_version: 1 };
      if (cmd === "get_engine_state") return mockEngineState;
      return [];
    });

    await waitFor(() => {
      expect(result.current.engineState).toEqual(mockEngineState);
    });

    // Simulate state update from backend (version bump)
    const newEngineState = { ...mockEngineState, is_running: true };
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "get_input_devices") return ["Mic 1"];
      if (cmd === "get_output_devices") return ["Speakers 1"];
      if (cmd === "get_metrics") return { ...mockMetrics, state_version: 2 };
      if (cmd === "get_engine_state") return newEngineState;
      return [];
    });

    // Wait for the polling interval to pick up the change
    await waitFor(
      () => {
        // We expect polling to trigger 'get_metrics' then 'get_engine_state'
        expect(result.current.engineState).toEqual(newEngineState);
        expect(result.current.isRunning).toBe(true);
      },
      { timeout: 2000 },
    );
  });
});

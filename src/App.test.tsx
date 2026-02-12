import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd) => {
    if (cmd === "get_input_devices") return ["Mic 1"];
    if (cmd === "get_output_devices") return ["Speakers 1"];
    if (cmd === "get_metrics")
      return {
        latency_ms: 10,
        cpu_usage: 5,
        input_level: 0.1,
        spectrum: Array(12).fill(0.1),
        tonality: Array(12).fill(0.1),
      };
    return {};
  }),
}));

describe("App Smoke Test", () => {
  it("renders without crashing", async () => {
    render(<App />);
    expect(screen.getByText(/LEYLINE/i)).toBeInTheDocument();
  });

  it("initializes with default expander settings", () => {
    render(<App />);
    expect(screen.getByText(/Noise Expander/i)).toBeInTheDocument();
  });

  it("displays N/A for latency when engine is not running", async () => {
    // Mock latency_ms as -1 to simulate engine not running
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "get_input_devices") return ["Mic 1"];
      if (cmd === "get_output_devices") return ["Speakers 1"];
      if (cmd === "get_metrics") {
        return {
          latency_ms: -1,
          cpu_usage: 2,
          input_level: 0,
          spectrum: Array(12).fill(0),
          tonality: Array(12).fill(0),
        };
      }
      return {};
    });

    render(<App />);
    expect(await screen.findByText(/N\/A/i)).toBeInTheDocument();
  });

  it("rounds latency to the nearest whole number", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "get_input_devices") return ["Mic 1"];
      if (cmd === "get_output_devices") return ["Speakers 1"];
      if (cmd === "get_metrics") {
        return {
          latency_ms: 10.7,
          cpu_usage: 5,
          input_level: 0,
          spectrum: Array(12).fill(0),
          tonality: Array(12).fill(0),
        };
      }
      return {};
    });

    render(<App />);
    expect(await screen.findByText(/11 ms/i)).toBeInTheDocument();
  });
});

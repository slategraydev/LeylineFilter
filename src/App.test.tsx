import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
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
});

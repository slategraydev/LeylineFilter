import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { Visualizer } from "./Visualizer";

describe("Visualizer", () => {
  it("renders the logo when not running", () => {
    render(
      <Visualizer
        isRunning={false}
        spectrum={Array(12).fill(0)}
        tonality={Array(12).fill(0)}
      />,
    );
    expect(screen.getByText(/READY/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Logo/i)).toBeInTheDocument();
  });

  it("renders bars and changes height based on spectrum when running", () => {
    const mockSpectrum = Array(12).fill(0.5);
    const mockTonality = Array(12).fill(0.5);
    const { container } = render(
      <Visualizer
        isRunning={true}
        spectrum={mockSpectrum}
        tonality={mockTonality}
      />,
    );
    const bars = container.querySelectorAll(".wave-bar");
    expect(bars.length).toBe(12);

    const heights = Array.from(bars).map((bar) =>
      parseFloat((bar as HTMLElement).style.height),
    );
    heights.forEach((h) => {
      expect(h).toBeGreaterThan(0);
    });
    expect(screen.queryByText(/READY/i)).not.toBeInTheDocument();
  });
});

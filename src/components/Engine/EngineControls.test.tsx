import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { EngineControls } from "./EngineControls";

describe("EngineControls", () => {
  const defaultProps = {
    isRunning: false,
    inputDevices: ["Mic 1", "Mic 2"],
    outputDevices: ["Speaker 1", "Speaker 2"],
    selectedInput: "Mic 1",
    selectedOutput: "Speaker 1",
    isMonitoring: false,
    onInputChange: vi.fn(),
    onOutputChange: vi.fn(),
    onMonitoringChange: vi.fn(),
  };

  it("renders input and output device selects", () => {
    render(<EngineControls {...defaultProps} />);
    expect(screen.getByLabelText(/^Input$/i)).not.toBeDisabled();
    expect(screen.getByLabelText(/^Output$/i)).not.toBeDisabled();
  });

  it("renders monitor toggle", () => {
    render(<EngineControls {...defaultProps} />);
    expect(screen.getByLabelText(/^Monitor$/i)).toBeInTheDocument();
  });

  it("calls onInputChange when input selection changes", () => {
    render(<EngineControls {...defaultProps} />);
    fireEvent.change(screen.getByLabelText(/^Input$/i), {
      target: { value: "Mic 2" },
    });
    expect(defaultProps.onInputChange).toHaveBeenCalledWith("Mic 2");
  });

  it("calls onOutputChange when output selection changes", () => {
    render(<EngineControls {...defaultProps} />);
    fireEvent.change(screen.getByLabelText(/^Output$/i), {
      target: { value: "Speaker 2" },
    });
    expect(defaultProps.onOutputChange).toHaveBeenCalledWith("Speaker 2");
  });

  it("calls onMonitoringChange when monitor toggle is clicked", () => {
    render(<EngineControls {...defaultProps} />);
    fireEvent.click(screen.getByLabelText(/^Monitor$/i));
    expect(defaultProps.onMonitoringChange).toHaveBeenCalledWith(true);
  });
});

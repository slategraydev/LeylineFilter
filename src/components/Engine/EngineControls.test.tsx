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
    selectedMonitor: "None",
    onInputChange: vi.fn(),
    onOutputChange: vi.fn(),
    onMonitorChange: vi.fn(),
  };

  it("renders all three device selects", () => {
    render(<EngineControls {...defaultProps} />);
    expect(screen.getByLabelText(/Microphone/i)).not.toBeDisabled();
    expect(screen.getByLabelText(/Destination/i)).not.toBeDisabled();
    expect(screen.getByLabelText(/Headphones/i)).not.toBeDisabled();
  });

  it("selects remain enabled while running for seamless swap", () => {
    render(<EngineControls {...defaultProps} isRunning={true} />);
    expect(screen.getByLabelText(/Microphone/i)).not.toBeDisabled();
    expect(screen.getByLabelText(/Destination/i)).not.toBeDisabled();
    expect(screen.getByLabelText(/Headphones/i)).not.toBeDisabled();
  });

  it("calls onInputChange when input selection changes", () => {
    render(<EngineControls {...defaultProps} />);
    fireEvent.change(screen.getByLabelText(/Microphone/i), {
      target: { value: "Mic 2" },
    });
    expect(defaultProps.onInputChange).toHaveBeenCalledWith("Mic 2");
  });

  it("calls onOutputChange when output selection changes", () => {
    render(<EngineControls {...defaultProps} />);
    fireEvent.change(screen.getByLabelText(/Destination/i), {
      target: { value: "Speaker 2" },
    });
    expect(defaultProps.onOutputChange).toHaveBeenCalledWith("Speaker 2");
  });

  it("calls onMonitorChange when monitor selection changes", () => {
    render(<EngineControls {...defaultProps} />);
    fireEvent.change(screen.getByLabelText(/Headphones/i), {
      target: { value: "Speaker 1" },
    });
    expect(defaultProps.onMonitorChange).toHaveBeenCalledWith("Speaker 1");
  });

  it("monitor defaults to None option", () => {
    render(<EngineControls {...defaultProps} />);
    expect(screen.getByLabelText(/Headphones/i)).toHaveValue("None");
  });
});

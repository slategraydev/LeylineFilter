import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { EngineControls } from "./EngineControls";

describe("EngineControls", () => {
  const defaultProps = {
    isRunning: false,
    isMonitoring: true,
    inputDevices: ["Mic 1", "Mic 2"],
    outputDevices: ["Speaker 1", "Speaker 2"],
    selectedInput: "Mic 1",
    selectedOutput: "Speaker 1",
    onInputChange: vi.fn(),
    onOutputChange: vi.fn(),
    onToggleMonitoring: vi.fn(),
  };

  it("renders correctly when stopped", () => {
    render(<EngineControls {...defaultProps} />);
    expect(screen.getByLabelText(/Input/i)).not.toBeDisabled();
    expect(screen.getByLabelText(/Output/i)).not.toBeDisabled();
    expect(screen.getByLabelText(/Monitor/i)).not.toBeDisabled();
  });

  it("renders correctly when running", () => {
    render(<EngineControls {...defaultProps} isRunning={true} />);
    // Selects should NOT be disabled anymore for seamless swap
    expect(screen.getByLabelText(/Input/i)).not.toBeDisabled();
    expect(screen.getByLabelText(/Output/i)).not.toBeDisabled();
  });

  it("calls onInputChange when input selection changes", () => {
    render(<EngineControls {...defaultProps} />);
    const select = screen.getByLabelText(/Input/i);
    fireEvent.change(select, { target: { value: "Mic 2" } });
    expect(defaultProps.onInputChange).toHaveBeenCalledWith("Mic 2");
  });

  it("calls onOutputChange when output selection changes", () => {
    render(<EngineControls {...defaultProps} />);
    const select = screen.getByLabelText(/Output/i);
    fireEvent.change(select, { target: { value: "Speaker 2" } });
    expect(defaultProps.onOutputChange).toHaveBeenCalledWith("Speaker 2");
  });
});

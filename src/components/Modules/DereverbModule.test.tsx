import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { DereverbModule } from "./DereverbModule";

describe("DereverbModule", () => {
  const defaultConfig = {
    enabled: true,
    reduction: 0.5,
    sensitivity: 0.5,
  };

  const defaultProps = {
    id: "test-dereverb",
    initialPosition: { gx: 1, gy: 1 },
    heightUnits: 15,
    widthUnits: 18,
    onPositionChange: () => {},
    onHeightReport: () => {},
    onUpdate: vi.fn(),
    onRemove: vi.fn(),
    onToggle: vi.fn(),
  };

  it("renders correctly with default config", () => {
    render(<DereverbModule {...defaultProps} config={defaultConfig} />);
    expect(screen.getByText(/Room De-Reverb/i)).toBeInTheDocument();
    expect(screen.getAllByText(/50%/i)).toHaveLength(2);
  });

  it("calls onUpdate when values are updated", () => {
    render(<DereverbModule {...defaultProps} config={defaultConfig} />);

    const reductionSlider = screen.getByLabelText(/Reduction/i);
    fireEvent.change(reductionSlider, { target: { value: "0.8" } });

    expect(defaultProps.onUpdate).toHaveBeenCalledWith(
      expect.objectContaining({
        reduction: 0.8,
      }),
    );
  });

  it("disables inputs when module is disabled", () => {
    const disabledConfig = { ...defaultConfig, enabled: false };
    render(<DereverbModule {...defaultProps} config={disabledConfig} />);

    const sliders = screen.getAllByRole("slider");
    sliders.forEach((slider) => {
      expect(slider).toBeDisabled();
    });
  });
});

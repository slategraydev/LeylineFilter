import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { SaturationModule } from './SaturationModule';

describe('SaturationModule', () => {
  const defaultConfig = {
    enabled: true,
    drive: 2.0,
    tilt: 0.0,
    mix: 0.5,
  };

  const defaultProps = {
    id: 'test-saturation',
    initialPosition: { gx: 1, gy: 1 },
    heightUnits: 22,
    widthUnits: 18,
    onPositionChange: () => {},
    onHeightReport: () => {},
  };

  it('renders correctly with default config', () => {
    render(<SaturationModule {...defaultProps} config={defaultConfig} onChange={() => {}} />);
    expect(screen.getByText(/Tube Saturation/i)).toBeInTheDocument();
    expect(screen.getByText(/2.0x/i)).toBeInTheDocument();
    expect(screen.getByText(/50%/i)).toBeInTheDocument();
  });

  it('calls onChange when values are updated', () => {
    const onChange = vi.fn();
    render(<SaturationModule {...defaultProps} config={defaultConfig} onChange={onChange} />);

    const driveSlider = screen.getByLabelText(/Drive/i);
    fireEvent.change(driveSlider, { target: { value: '5.0' } });

    expect(onChange).toHaveBeenCalledWith(
      expect.objectContaining({
        drive: 5.0,
      }),
    );
  });

  it('disables inputs when module is disabled', () => {
    const disabledConfig = { ...defaultConfig, enabled: false };
    render(<SaturationModule {...defaultProps} config={disabledConfig} onChange={() => {}} />);

    const sliders = screen.getAllByRole('slider');
    sliders.forEach((slider) => {
      expect(slider).toBeDisabled();
    });
  });
});

import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { DeesserModule } from './DeesserModule';

describe('DeesserModule', () => {
  const defaultConfig = {
    enabled: true,
    threshold_db: -20.0,
    frequency: 6000.0,
    ratio: 4.0,
    attack_ms: 5.0,
    release_ms: 50.0,
  };

  const defaultProps = {
    id: 'test-deesser',
    initialPosition: { gx: 1, gy: 1 },
    heightUnits: 22,
    widthUnits: 18,
    onPositionChange: () => {},
    onHeightReport: () => {},
  };

  it('renders correctly with default config', () => {
    render(<DeesserModule {...defaultProps} config={defaultConfig} onChange={() => {}} />);
    expect(screen.getByText(/Vocal De-Esser/i)).toBeInTheDocument();
    expect(screen.getByText(/-20.0 dB/i)).toBeInTheDocument();
    expect(screen.getByText(/6000 Hz/i)).toBeInTheDocument();
  });

  it('calls onChange when values are updated', () => {
    const onChange = vi.fn();
    render(<DeesserModule {...defaultProps} config={defaultConfig} onChange={onChange} />);

    const thresholdSlider = screen.getByLabelText(/Threshold/i);
    fireEvent.change(thresholdSlider, { target: { value: '-10.5' } });

    expect(onChange).toHaveBeenCalledWith(
      expect.objectContaining({
        threshold_db: -10.5,
      }),
    );
  });

  it('disables inputs when module is disabled', () => {
    const disabledConfig = { ...defaultConfig, enabled: false };
    render(<DeesserModule {...defaultProps} config={disabledConfig} onChange={() => {}} />);

    const sliders = screen.getAllByRole('slider');
    sliders.forEach((slider) => {
      expect(slider).toBeDisabled();
    });
  });
});

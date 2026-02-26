import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { LimiterModule } from './LimiterModule';

describe('LimiterModule', () => {
  const defaultConfig = {
    enabled: true,
    threshold_db: -6.0,
    release_ms: 100,
  };

  const defaultProps = {
    id: "test-limiter",
    initialPosition: { gx: 1, gy: 1 },
    heightUnits: 12,
    widthUnits: 18,
    onPositionChange: () => { },
    onHeightReport: () => { },
    onUpdate: vi.fn(),
    onRemove: vi.fn(),
    onToggle: vi.fn(),
  };

  it('renders correctly with default config', () => {
    render(<LimiterModule {...defaultProps} config={defaultConfig} />);
    expect(screen.getByText(/Brickwall Limiter/i)).toBeInTheDocument();
    expect(screen.getByText(/-6.0 dB/i)).toBeInTheDocument();
  });

  it('calls onUpdate when values are updated', () => {
    render(<LimiterModule {...defaultProps} config={defaultConfig} />);

    const thresholdSlider = screen.getByLabelText(/Threshold/i);
    fireEvent.change(thresholdSlider, { target: { value: '-12.5' } });

    expect(defaultProps.onUpdate).toHaveBeenCalledWith(expect.objectContaining({
      threshold_db: -12.5
    }));
  });

  it('disables inputs when module is disabled', () => {
    const disabledConfig = { ...defaultConfig, enabled: false };
    render(<LimiterModule {...defaultProps} config={disabledConfig} />);

    const sliders = screen.getAllByRole('slider');
    sliders.forEach(slider => {
      expect(slider).toBeDisabled();
    });
  });
});

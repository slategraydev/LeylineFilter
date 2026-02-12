import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { GainModule } from './GainModule';

describe('GainModule', () => {
  const defaultConfig = {
    enabled: true,
    gain_db: 0.0,
  };

  const defaultProps = {
    id: "test-gain",
    initialPosition: { gx: 1, gy: 1 },
    heightUnits: 12,
    onPositionChange: () => { },
  };

  it('renders correctly with default config', () => {
    render(<GainModule {...defaultProps} config={defaultConfig} onChange={() => { }} />);
    expect(screen.getByText(/Master Gain/i)).toBeInTheDocument();
    expect(screen.getByText(/0.0 dB/i)).toBeInTheDocument();
  });

  it('calls onChange when values are updated', () => {
    const onChange = vi.fn();
    render(<GainModule {...defaultProps} config={defaultConfig} onChange={onChange} />);

    const gainSlider = screen.getByRole('slider');
    fireEvent.change(gainSlider, { target: { value: '5.5' } });

    expect(onChange).toHaveBeenCalledWith(expect.objectContaining({
      gain_db: 5.5
    }));
  });

  it('disables inputs when module is disabled', () => {
    const disabledConfig = { ...defaultConfig, enabled: false };
    render(<GainModule {...defaultProps} config={disabledConfig} onChange={() => { }} />);

    expect(screen.getByRole('slider')).toBeDisabled();
  });
});

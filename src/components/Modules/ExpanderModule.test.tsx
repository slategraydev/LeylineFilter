import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { ExpanderModule } from './ExpanderModule';

describe('ExpanderModule', () => {
  const defaultConfig = {
    enabled: true,
    threshold: 0.08,
    ratio: 2.0,
    attack_ms: 10.0,
    release_ms: 100.0,
  };

  it('renders correctly with default config', () => {
    render(<ExpanderModule config={defaultConfig} onChange={() => {}} />);
    expect(screen.getByText(/Noise Expander/i)).toBeInTheDocument();
    expect(screen.getByText(/Ratio: 2.0:1/i)).toBeInTheDocument();
  });

  it('calls onChange when values are updated', () => {
    const onChange = vi.fn();
    render(<ExpanderModule config={defaultConfig} onChange={onChange} />);

    const thresholdSlider = screen.getByLabelText(/Threshold/i);
    fireEvent.change(thresholdSlider, { target: { value: '0.1' } });

    expect(onChange).toHaveBeenCalledWith(expect.objectContaining({
      threshold: 0.1
    }));
  });

  it('disables inputs when module is disabled', () => {
    const disabledConfig = { ...defaultConfig, enabled: false };
    render(<ExpanderModule config={disabledConfig} onChange={() => {}} />);

    const sliders = screen.getAllByRole('slider');
    sliders.forEach(slider => {
      expect(slider).toBeDisabled();
    });
  });
});

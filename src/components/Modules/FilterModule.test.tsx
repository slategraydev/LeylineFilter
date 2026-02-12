import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { FilterModule } from './FilterModule';

describe('FilterModule', () => {
  const defaultConfig = {
    enabled: true,
    filter_type: 'LPF' as const,
    frequency: 1000.0,
    q: 1.0,
  };

  it('renders correctly with default config', () => {
    render(<FilterModule config={defaultConfig} onChange={() => { }} />);
    expect(screen.getByText(/Audio Filter/i)).toBeInTheDocument();
    expect(screen.getByText(/1000 Hz/i)).toBeInTheDocument();
  });

  it('calls onChange when values are updated', () => {
    const onChange = vi.fn();
    render(<FilterModule config={defaultConfig} onChange={onChange} />);

    const freqSlider = screen.getByLabelText(/Frequency/i);
    fireEvent.change(freqSlider, { target: { value: '500' } });

    expect(onChange).toHaveBeenCalledWith(expect.objectContaining({
      frequency: 500
    }));
  });

  it('updates filter type', () => {
    const onChange = vi.fn();
    render(<FilterModule config={defaultConfig} onChange={onChange} />);

    const typeSelect = screen.getByRole('combobox');
    fireEvent.change(typeSelect, { target: { value: 'HPF' } });

    expect(onChange).toHaveBeenCalledWith(expect.objectContaining({
      filter_type: 'HPF'
    }));
  });

  it('disables inputs when module is disabled', () => {
    const disabledConfig = { ...defaultConfig, enabled: false };
    render(<FilterModule config={disabledConfig} onChange={() => { }} />);

    const sliders = screen.getAllByRole('slider');
    sliders.forEach(slider => {
      expect(slider).toBeDisabled();
    });

    expect(screen.getByRole('combobox')).toBeDisabled();
  });
});

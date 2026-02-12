import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { RNNoiseModule } from './RNNoiseModule';

describe('RNNoiseModule', () => {
  const defaultConfig = {
    enabled: true,
  };

  it('renders correctly with default config', () => {
    render(<RNNoiseModule config={defaultConfig} onChange={() => { }} />);
    expect(screen.getByText(/Noise Suppression/i)).toBeInTheDocument();
  });

  it('toggles enabled state', () => {
    const onChange = vi.fn();
    render(<RNNoiseModule config={defaultConfig} onChange={onChange} />);

    // BaseModule renders a checkbox for enabling/disabling
    const checkbox = screen.getByRole('checkbox');
    fireEvent.click(checkbox);

    expect(onChange).toHaveBeenCalledWith(expect.objectContaining({
      enabled: false
    }));
  });

  it('displays description text', () => {
    render(<RNNoiseModule config={defaultConfig} onChange={() => { }} />);
    expect(screen.getByText(/Recurrent Neural Networks/i)).toBeInTheDocument();
  });
});

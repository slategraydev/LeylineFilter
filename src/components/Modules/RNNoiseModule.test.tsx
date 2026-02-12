import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { RNNoiseModule } from './RNNoiseModule';

describe('RNNoiseModule', () => {
  const defaultConfig = {
    enabled: true,
  };

  const defaultProps = {
    id: "test-rnnoise",
    initialPosition: { gx: 1, gy: 1 },
    heightUnits: 11,
    onPositionChange: () => { },
  };

  it('renders correctly with default config', () => {
    render(<RNNoiseModule {...defaultProps} config={defaultConfig} onChange={() => { }} />);
    expect(screen.getByText(/Noise Suppression/i)).toBeInTheDocument();
  });

  it('toggles enabled state', () => {
    const onChange = vi.fn();
    render(<RNNoiseModule {...defaultProps} config={defaultConfig} onChange={onChange} />);

    // BaseModule renders a checkbox for enabling/disabling
    const checkbox = screen.getByRole('checkbox');
    fireEvent.click(checkbox);

    expect(onChange).toHaveBeenCalledWith(expect.objectContaining({
      enabled: false
    }));
  });

  it('displays description text', () => {
    render(<RNNoiseModule {...defaultProps} config={defaultConfig} onChange={() => { }} />);
    expect(screen.getByText(/Recurrent Neural Networks/i)).toBeInTheDocument();
  });
});

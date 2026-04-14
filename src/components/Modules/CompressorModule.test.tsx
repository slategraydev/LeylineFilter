import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { CompressorModule } from './CompressorModule';

describe('CompressorModule', () => {
  const defaultProps = {
    id: 'comp-1',
    initialPosition: { gx: 1, gy: 1 },
    heightUnits: 6,
    widthUnits: 18,
    onPositionChange: vi.fn(),
    config: {
      enabled: true,
      threshold_db: -20,
      ratio: 4,
      attack_ms: 10,
      release_ms: 100,
      knee_db: 6,
      makeup_gain_db: 0,
    },
    onChange: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
    // Mock ResizeObserver
    vi.stubGlobal(
      'ResizeObserver',
      class {
        observe = vi.fn();
        unobserve = vi.fn();
        disconnect = vi.fn();
      },
    );
  });

  it('renders correctly', () => {
    render(<CompressorModule {...defaultProps} />);
    expect(screen.getByText('Dynamic Compressor')).toBeInTheDocument();
    expect(screen.getByText(/Threshold/)).toBeInTheDocument();
    expect(screen.getByText(/-20.0 dB/)).toBeInTheDocument();
    expect(screen.getByText(/Ratio/)).toBeInTheDocument();
    expect(screen.getByText(/4.0:1/)).toBeInTheDocument();
  });

  it('triggers onChange when threshold is changed', () => {
    render(<CompressorModule {...defaultProps} />);
    const thresholdInput = screen.getByLabelText(/Threshold/);
    fireEvent.change(thresholdInput, { target: { value: '-10' } });
    expect(defaultProps.onChange).toHaveBeenCalledWith({
      ...defaultProps.config,
      threshold_db: -10,
    });
  });

  it('triggers onChange when ratio is changed', () => {
    render(<CompressorModule {...defaultProps} />);
    const ratioInput = screen.getByLabelText(/Ratio/);
    fireEvent.change(ratioInput, { target: { value: '8' } });
    expect(defaultProps.onChange).toHaveBeenCalledWith({
      ...defaultProps.config,
      ratio: 8,
    });
  });

  it('disables inputs when config is disabled', () => {
    render(
      <CompressorModule {...defaultProps} config={{ ...defaultProps.config, enabled: false }} />,
    );
    const thresholdInput = screen.getByLabelText(/Threshold/) as HTMLInputElement;
    expect(thresholdInput.disabled).toBe(true);
  });
});

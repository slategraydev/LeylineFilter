import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ParametricEQModule } from './ParametricEQModule';

describe('ParametricEQModule', () => {
  const defaultProps = {
    id: 'eq-1',
    initialPosition: { gx: 1, gy: 1 },
    heightUnits: 12,
    widthUnits: 18,
    onPositionChange: vi.fn(),
    config: {
      enabled: true,
      bands: [
        { enabled: true, filter_type: 'LowShelf' as const, frequency: 100, q: 0.707, gain_db: 0 },
        { enabled: true, filter_type: 'Peaking' as const, frequency: 1000, q: 1, gain_db: 0 },
        { enabled: true, filter_type: 'HighShelf' as const, frequency: 8000, q: 0.707, gain_db: 0 },
      ],
    },
    onChange: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal(
      'ResizeObserver',
      class {
        observe = vi.fn();
        unobserve = vi.fn();
        disconnect = vi.fn();
      },
    );
  });

  it('renders all bands', () => {
    render(<ParametricEQModule {...defaultProps} />);
    expect(screen.getByText('Parametric EQ')).toBeInTheDocument();
    expect(screen.getByText('Low Shelf')).toBeInTheDocument();
    expect(screen.getByText('High Shelf')).toBeInTheDocument();
    expect(screen.getByText('Mid 1')).toBeInTheDocument();
  });

  it('triggers onChange when a band frequency is changed', () => {
    render(<ParametricEQModule {...defaultProps} />);
    // Get the first frequency input (Low Shelf)
    const freqInputs = screen.getAllByLabelText(/Freq/);
    fireEvent.change(freqInputs[0], { target: { value: '150' } });

    expect(defaultProps.onChange).toHaveBeenCalled();
    const callArgs = defaultProps.onChange.mock.calls[0][0];
    expect(callArgs.bands[0].frequency).toBe(150);
  });

  it('disables inputs when individual band is disabled', () => {
    const disabledConfig = { ...defaultProps.config };
    disabledConfig.bands[0].enabled = false;

    render(<ParametricEQModule {...defaultProps} config={disabledConfig} />);
    const freqInputs = screen.getAllByLabelText(/Freq/) as HTMLInputElement[];
    expect(freqInputs[0].disabled).toBe(true);
    expect(freqInputs[1].disabled).toBe(false);
  });
});

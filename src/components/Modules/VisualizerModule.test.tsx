// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { VisualizerModule } from './VisualizerModule';

describe('VisualizerModule', () => {
  const defaultProps = {
    id: "test-viz",
    initialPosition: { gx: 1, gy: 1 },
    heightUnits: 16,
    widthUnits: 18,
    onPositionChange: () => { },
    onHeightReport: () => { },
    enabled: true,
    onToggle: vi.fn(),
    isRunning: true,
    spectrum: Array(12).fill(0.5),
    tonality: Array(12).fill(0.5),
  };

  it('renders the module title', () => {
    render(<VisualizerModule {...defaultProps} />);
    expect(screen.getByText('Spectrum Analyzer')).toBeInTheDocument();
  });

  it('calls onToggle when the switch is clicked', () => {
    render(<VisualizerModule {...defaultProps} />);
    const toggle = screen.getByRole('checkbox');
    fireEvent.click(toggle);
    expect(defaultProps.onToggle).toHaveBeenCalledWith(false);
  });

  it('renders the visualizer bars when running', () => {
    const { container } = render(<VisualizerModule {...defaultProps} />);
    // Wave bars have the class 'wave-bar'
    const bars = container.querySelectorAll('.wave-bar');
    expect(bars.length).toBe(12);
  });

  it('renders the logo when not running', () => {
    render(<VisualizerModule {...defaultProps} isRunning={false} />);
    // LeylineLogo has an aria-label "Logo"
    expect(screen.getByLabelText('Logo')).toBeInTheDocument();
  });
});

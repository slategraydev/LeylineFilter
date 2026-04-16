import { render, screen, waitFor, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import App from './App';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('App Smoke Test', () => {
  const defaultState = {
    modules: [
      {
        id: 'expander-1',
        name: 'Expander',
        category: 'Dynamics',
        enabled: true,
        config: {
          type: 'Expander',
          data: {
            enabled: true,
            threshold: 0.1,
            ratio: 2.0,
            attack_ms: 10,
            release_ms: 100,
          },
        },
      },
    ],
    is_running: false,
    monitoring_enabled: false,
    sample_rate: 48000,
  };

  const defaultMetrics = {
    latency_ms: 10,
    cpu_usage: 5,
    input_level: 0.1,
    spectrum: Array(12).fill(0.1),
    tonality: Array(12).fill(0.1),
    state_version: 0,
  };

  beforeEach(() => {
    localStorage.clear();
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === 'get_input_devices') return ['Mic 1'];
      if (cmd === 'get_output_devices') return ['Speakers 1'];
      if (cmd === 'get_engine_state') return defaultState;
      if (cmd === 'get_metrics') return defaultMetrics;
      return {};
    });
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('renders without crashing', async () => {
    await act(async () => {
      render(<App />);
    });
    expect(screen.getByText(/LEYLINE/i)).toBeInTheDocument();
  });

  it('initializes with default expander settings', async () => {
    await act(async () => {
      render(<App />);
    });
    expect(await screen.findByText(/Noise Expander/i)).toBeInTheDocument();
  });

  it('displays 0 ms for latency when engine is not running', async () => {
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === 'get_input_devices') return ['Mic 1'];
      if (cmd === 'get_output_devices') return ['Speakers 1'];
      if (cmd === 'get_engine_state') return { modules: [], is_running: false, sample_rate: 48000 };
      if (cmd === 'get_metrics') {
        return {
          ...defaultMetrics,
          latency_ms: 0,
        };
      }
      return {};
    });

    await act(async () => {
      render(<App />);
    });

    const latencyDisplay = await screen.findByTestId('latency-value');
    await waitFor(() => expect(latencyDisplay).toHaveTextContent('0 ms'));
  });

  it('renders the engine toggle button', async () => {
    await act(async () => {
      render(<App />);
    });
    expect(screen.getByText(/Start Engine/i)).toBeInTheDocument();
  });

  it('rounds latency to the nearest whole number', async () => {
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === 'get_input_devices') return ['Mic 1'];
      if (cmd === 'get_output_devices') return ['Speakers 1'];
      if (cmd === 'get_engine_state') return defaultState;
      if (cmd === 'get_metrics') {
        return {
          ...defaultMetrics,
          latency_ms: 10.7,
        };
      }
      return {};
    });

    await act(async () => {
      render(<App />);
    });

    const latencyDisplay = await screen.findByTestId('latency-value');
    await waitFor(() => expect(latencyDisplay).toHaveTextContent('11 ms'));
  });

  it('syncs layout and persists to disk when positions change', async () => {
    // Mock get_engine_state to return a module
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === 'get_input_devices') return ['Mic 1'];
      if (cmd === 'get_output_devices') return ['Speakers 1'];
      if (cmd === 'get_engine_state') return defaultState;
      if (cmd === 'get_metrics') return defaultMetrics;
      return {};
    });

    await act(async () => {
      render(<App />);
    });

    // Wait for initial layout sync (which now triggers save_to_disk in backend)
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('update_layout', expect.any(Object));
    });
  });
});

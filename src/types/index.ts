// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// GLOBAL TYPES
// Core interfaces and type definitions for the application state.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

export interface ExpanderConfig {
  enabled: boolean;
  threshold: number;
  ratio: number;
  attack_ms: number;
  release_ms: number;
}

export interface RNNoiseConfig {
  enabled: boolean;
}

export interface GainConfig {
  enabled: boolean;
  gain_db: number;
}

export interface CompressorConfig {
  enabled: boolean;
  threshold_db: number;
  ratio: number;
  attack_ms: number;
  release_ms: number;
  knee_db: number;
  makeup_gain_db: number;
}

export interface FilterConfig {
  enabled: boolean;
  filter_type: 'HPF' | 'LPF' | 'BPF' | 'Notch';
  frequency: number;
  q: number;
}

export interface EQBandConfig {
  enabled: boolean;
  filter_type: 'LowShelf' | 'HighShelf' | 'Peaking';
  frequency: number;
  q: number;
  gain_db: number;
}

export interface ParametricEQConfig {
  enabled: boolean;
  bands: EQBandConfig[];
}

export interface FXConfig {
  enabled: boolean;
  fx_type: 'Reverb' | 'Delay' | 'Chorus' | 'Flanger';
  mix: number;
  params: Record<string, number>;
}

export interface DeesserConfig {
  enabled: boolean;
  threshold_db: number;
  ratio: number;
  attack_ms: number;
  release_ms: number;
  frequency: number;
}

export interface SaturationConfig {
  enabled: boolean;
  drive: number;
  tilt: number;
  mix: number;
}

export interface LimiterConfig {
  enabled: boolean;
  threshold_db: number;
  release_ms: number;
}

export interface DereverbConfig {
  enabled: boolean;
  reduction: number;
  sensitivity: number;
}

export interface EngineMetrics {
  latency_ms: number;
  cpu_usage: number;
  input_level: number;
  input_level_db: number;
  buffer_size: number;
  spectrum: number[];
  tonality: number[];
  waveform: number[];
  state_version: number;
}

export interface ModuleInfo {
  id: string;
  name: string;
  category: 'Dynamics' | 'Filter' | 'Voice' | 'FX' | 'Synth' | 'Utility';
  enabled: boolean;
  config: ModuleConfig;
}

export interface GridPosition {
  gx: number;
  gy: number;
}

export interface EngineState {
  modules: ModuleInfo[];
  is_running: boolean;
  monitoring_enabled: boolean;
  sample_rate: number;
  buffer_size: number;
  input_device: string | null;
  output_device: string | null;
  positions: Record<string, GridPosition>;
  heights: Record<string, number>;
  widths: Record<string, number>;
}

export type ModuleConfig =
  | { type: 'Expander'; data: ExpanderConfig }
  | { type: 'RNNoise'; data: RNNoiseConfig }
  | { type: 'Gain'; data: GainConfig }
  | { type: 'Compressor'; data: CompressorConfig }
  | { type: 'Filter'; data: FilterConfig }
  | { type: 'ParametricEQ'; data: ParametricEQConfig }
  | { type: 'Deesser'; data: DeesserConfig }
  | { type: 'Saturation'; data: SaturationConfig }
  | { type: 'Limiter'; data: LimiterConfig }
  | { type: 'Dereverb'; data: DereverbConfig }
  | { type: 'FX'; data: FXConfig }
  | { type: 'None'; data: null };

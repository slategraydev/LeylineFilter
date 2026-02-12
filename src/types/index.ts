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
  filter_type: "HPF" | "LPF" | "BPF" | "Notch";
  frequency: number;
  q: number;
}

export interface VisualizerConfig {
  enabled: boolean;
}

export interface FXConfig {
  enabled: boolean;
  fx_type: "Reverb" | "Delay" | "Chorus" | "Flanger";
  mix: number;
  params: Record<string, number>;
}

export interface EngineMetrics {
  latency_ms: number;
  cpu_usage: number;
  input_level: number;
  buffer_size: number;
  spectrum: number[];
  tonality: number[];
  state_version: number;
}

export interface ModuleInfo {
  id: string;
  name: string;
  category: "Dynamics" | "Filter" | "Voice" | "FX" | "Synth" | "Utility";
  enabled: boolean;
  config: ModuleConfig;
}

export interface EngineState {
  modules: ModuleInfo[];
  is_running: boolean;
  sample_rate: number;
  buffer_size: number;
}

export type ModuleConfig =
  | { type: "Expander"; data: ExpanderConfig }
  | { type: "RNNoise"; data: RNNoiseConfig }
  | { type: "Gain"; data: GainConfig }
  | { type: "Compressor"; data: CompressorConfig }
  | { type: "Filter"; data: FilterConfig }
  | { type: "FX"; data: FXConfig }
  | { type: "Visualizer"; data: VisualizerConfig }
  | { type: "None"; data: null };

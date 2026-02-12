export interface ExpanderConfig {
  enabled: boolean;
  threshold: number;
  ratio: number;
  attack_ms: number;
  release_ms: number;
}

export interface EngineMetrics {
  latency_ms: number;
  cpu_usage: number;
  input_level: number;
  spectrum: number[];
  tonality: number[];
}

export type ModuleConfig = { type: "Expander"; data: ExpanderConfig };

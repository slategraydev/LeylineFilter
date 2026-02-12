import { RNNoiseConfig } from "../../types";
import { BaseModule } from "./BaseModule";

interface RNNoiseModuleProps {
  title?: string;
  config: RNNoiseConfig;
  onChange: (config: RNNoiseConfig) => void;
}

export function RNNoiseModule({
  title = "Noise Suppression (RNNoise)",
  config,
  onChange
}: RNNoiseModuleProps) {
  const updateConfig = (updates: Partial<RNNoiseConfig>) => {
    onChange({ ...config, ...updates });
  };

  return (
    <BaseModule
      title={title}
      enabled={config.enabled}
      onToggle={(enabled) => updateConfig({ enabled })}
    >
      <div className="control-group">
        <p style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', margin: '0 0 1rem 0' }}>
          Uses Recurrent Neural Networks to suppress non-stationary noise in real-time.
        </p>
      </div>
    </BaseModule>
  );
}

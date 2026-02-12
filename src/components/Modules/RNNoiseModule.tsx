import { RNNoiseConfig } from "../../types";
import { BaseModule } from "./BaseModule";
import { GridPosition } from "../../hooks/useDraggable";

interface RNNoiseModuleProps {
  id: string;
  initialPosition: GridPosition;
  heightUnits: number;
  onPositionChange: (id: string, pos: GridPosition) => void;
  title?: string;
  config: RNNoiseConfig;
  onChange: (config: RNNoiseConfig) => void;
  onRemove?: () => void;
  style?: React.CSSProperties;
}

export function RNNoiseModule({
  id,
  initialPosition,
  heightUnits,
  onPositionChange,
  title = "Noise Suppression",
  config,
  onChange,
  onRemove,
  style,
}: RNNoiseModuleProps) {
  const updateConfig = (updates: Partial<RNNoiseConfig>) => {
    onChange({ ...config, ...updates });
  };

  return (
    <BaseModule
      id={id}
      initialPosition={initialPosition}
      heightUnits={heightUnits}
      onPositionChange={onPositionChange}
      title={title}
      enabled={config.enabled}
      onToggle={(enabled) => updateConfig({ enabled })}
      onRemove={onRemove}
      style={style}
    >
      <div className="control-group">
        <p style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', margin: '0 0 1rem 0' }}>
          Uses Recurrent Neural Networks to suppress non-stationary noise in real-time.
        </p>
      </div>
    </BaseModule>
  );
}

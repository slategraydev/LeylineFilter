import { RNNoiseConfig } from "../../types";
import { BaseModule } from "./BaseModule";
import { GridPosition } from "../../hooks/useDraggable";

interface RNNoiseModuleProps {
  id: string;
  initialPosition: GridPosition;
  heightUnits: number;
  widthUnits: number;
  onPositionChange: (id: string, pos: GridPosition) => void;
  onDrag?: (id: string, pos: GridPosition | null, rawOffset?: { x: number, y: number }, continuousPos?: GridPosition) => void;
  onHeightReport?: (id: string, units: number) => void;
  onWidthReport?: (id: string, units: number) => void;
  title?: string;
  config: RNNoiseConfig;
  onChange: (config: RNNoiseConfig) => void;
  onRemove?: () => void;
  style?: React.CSSProperties;
  isNewlyPlaced?: boolean;
}

export function RNNoiseModule({
  id,
  initialPosition,
  heightUnits,
  widthUnits,
  onPositionChange,
  onDrag,
  onHeightReport,
  onWidthReport,
  title = "Noise Suppression",
  config,
  onChange,
  onRemove,
  style,
  isNewlyPlaced,
}: RNNoiseModuleProps) {
  const updateConfig = (updates: Partial<RNNoiseConfig>) => {
    onChange({ ...config, ...updates });
  };

  return (
    <BaseModule
      id={id}
      initialPosition={initialPosition}
      heightUnits={heightUnits}
      widthUnits={widthUnits}
      onPositionChange={onPositionChange}
      onDrag={onDrag}
      onHeightReport={onHeightReport}
      onWidthReport={onWidthReport}
      title={title}
      enabled={config.enabled}
      onToggle={(enabled) => updateConfig({ enabled })}
      onRemove={onRemove}
      style={style}
      isNewlyPlaced={isNewlyPlaced}
    >
      <div className="control-group">
        <p style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', margin: 0 }}>
          Uses Recurrent Neural Networks to suppress non-stationary noise in real-time.
        </p>
      </div>
    </BaseModule>
  );
}

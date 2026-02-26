import React from "react";
import { BaseModule } from "./BaseModule";
import { LimiterConfig, GridPosition } from "../../types";

interface LimiterModuleProps {
  id: string;
  initialPosition: GridPosition;
  heightUnits: number;
  widthUnits: number;
  onPositionChange: (id: string, pos: GridPosition) => void;
  onDrag?: (
    id: string,
    pos: GridPosition | null,
    rawOffset?: { x: number; y: number },
    continuousPos?: GridPosition,
  ) => void;
  onHeightReport?: (id: string, h: number) => void;
  onWidthReport?: (id: string, units: number) => void;
  config: LimiterConfig;
  onUpdate: (config: LimiterConfig) => void;
  onRemove: () => void;
  onToggle: (enabled: boolean) => void;
  style?: React.CSSProperties;
  isNewlyPlaced?: boolean;
}

export const LimiterModule: React.FC<LimiterModuleProps> = ({
  id,
  initialPosition,
  heightUnits,
  widthUnits,
  onPositionChange,
  onDrag,
  onHeightReport,
  onWidthReport,
  config,
  onUpdate,
  onRemove,
  onToggle,
  isNewlyPlaced,
}) => {
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
      title="Brickwall Limiter"
      enabled={config.enabled}
      onToggle={onToggle}
      onRemove={onRemove}
      isNewlyPlaced={isNewlyPlaced}
    >
      <p className="module-description">
        Hard-caps peaks to ensure zero digital clipping.
      </p>
      <div className="control-group">
        <label>
          Threshold <span>{config.threshold_db.toFixed(1)} dB</span>
          <input
            type="range"
            min="-60"
            max="0"
            step="0.1"
            value={config.threshold_db}
            disabled={!config.enabled}
            onChange={(e) =>
              onUpdate({ ...config, threshold_db: parseFloat(e.target.value) })
            }
          />
        </label>
      </div>

      <div className="control-group">
        <label>
          Release <span>{config.release_ms} ms</span>
          <input
            type="range"
            min="1"
            max="1000"
            step="1"
            value={config.release_ms}
            disabled={!config.enabled}
            onChange={(e) =>
              onUpdate({ ...config, release_ms: parseInt(e.target.value) })
            }
          />
        </label>
      </div>
    </BaseModule>
  );
};

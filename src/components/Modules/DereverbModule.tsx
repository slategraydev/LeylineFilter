import React from "react";
import { BaseModule } from "./BaseModule";
import { DereverbConfig, GridPosition } from "../../types";

interface DereverbModuleProps {
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
  config: DereverbConfig;
  onUpdate: (config: DereverbConfig) => void;
  onRemove: () => void;
  onToggle: (enabled: boolean) => void;
  style?: React.CSSProperties;
  isNewlyPlaced?: boolean;
}

export const DereverbModule: React.FC<DereverbModuleProps> = ({
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
      title="Room De-Reverb"
      enabled={config.enabled}
      onToggle={onToggle}
      onRemove={onRemove}
      isNewlyPlaced={isNewlyPlaced}
    >
      <p className="module-description">
        Removes room resonance and tightens vocal transients.
      </p>
      <div className="control-group">
        <label>
          Reduction <span>{Math.round(config.reduction * 100)}%</span>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={config.reduction}
            disabled={!config.enabled}
            onChange={(e) =>
              onUpdate({ ...config, reduction: parseFloat(e.target.value) })
            }
          />
        </label>
      </div>

      <div className="control-group">
        <label>
          Sensitivity <span>{Math.round(config.sensitivity * 100)}%</span>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={config.sensitivity}
            disabled={!config.enabled}
            onChange={(e) =>
              onUpdate({ ...config, sensitivity: parseFloat(e.target.value) })
            }
          />
        </label>
      </div>
    </BaseModule>
  );
};

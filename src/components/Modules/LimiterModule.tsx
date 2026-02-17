import React from 'react';
import { BaseModule } from './BaseModule';
import { LimiterConfig } from '../../types';
import { GridPosition } from '../../hooks/useDraggable';

interface LimiterModuleProps {
    id: string;
    initialPosition: GridPosition;
    heightUnits: number;
    widthUnits: number;
    onPositionChange: (id: string, pos: GridPosition) => void;
    onDrag?: (id: string, pos: GridPosition | null) => void;
    onHeightReport?: (id: string, h: number) => void;
    config: LimiterConfig;
    onUpdate: (config: LimiterConfig) => void;
    onRemove: () => void;
    onToggle: (enabled: boolean) => void;
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
            title="Brickwall Limiter"
            enabled={config.enabled}
            onToggle={onToggle}
            onRemove={onRemove}
            isNewlyPlaced={isNewlyPlaced}
        >
            <div className="module-controls">
                <label className="control-group">
                    <div className="control-header">
                        <span>Threshold</span>
                        <span className="value">{config.threshold_db.toFixed(1)} dB</span>
                    </div>
                    <input
                        type="range"
                        min="-60"
                        max="0"
                        step="0.1"
                        value={config.threshold_db}
                        onChange={(e) => onUpdate({ ...config, threshold_db: parseFloat(e.target.value) })}
                    />
                </label>

                <label className="control-group">
                    <div className="control-header">
                        <span>Release</span>
                        <span className="value">{config.release_ms} ms</span>
                    </div>
                    <input
                        type="range"
                        min="1"
                        max="1000"
                        step="1"
                        value={config.release_ms}
                        onChange={(e) => onUpdate({ ...config, release_ms: parseInt(e.target.value) })}
                    />
                </label>

                <div className="module-footer-note">
                    Built-in 5ms lookahead for zero-clip safety.
                </div>
            </div>
        </BaseModule>
    );
};

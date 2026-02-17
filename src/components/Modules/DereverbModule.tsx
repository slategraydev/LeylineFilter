import React from 'react';
import { BaseModule } from './BaseModule';
import { DereverbConfig } from '../../types';
import { GridPosition } from '../../hooks/useDraggable';

interface DereverbModuleProps {
    id: string;
    initialPosition: GridPosition;
    heightUnits: number;
    widthUnits: number;
    onPositionChange: (id: string, pos: GridPosition) => void;
    onDrag?: (id: string, pos: GridPosition | null) => void;
    onHeightReport?: (id: string, h: number) => void;
    config: DereverbConfig;
    onUpdate: (config: DereverbConfig) => void;
    onRemove: () => void;
    onToggle: (enabled: boolean) => void;
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
            title="Room De-Reverb"
            enabled={config.enabled}
            onToggle={onToggle}
            onRemove={onRemove}
            isNewlyPlaced={isNewlyPlaced}
        >
            <div className="module-controls">
                <label className="control-group">
                    <div className="control-header">
                        <span>Reduction</span>
                        <span className="value">{Math.round(config.reduction * 100)}%</span>
                    </div>
                    <input
                        type="range"
                        min="0"
                        max="1"
                        step="0.01"
                        value={config.reduction}
                        onChange={(e) => onUpdate({ ...config, reduction: parseFloat(e.target.value) })}
                    />
                </label>

                <label className="control-group">
                    <div className="control-header">
                        <span>Sensitivity</span>
                        <span className="value">{Math.round(config.sensitivity * 100)}%</span>
                    </div>
                    <input
                        type="range"
                        min="0"
                        max="1"
                        step="0.01"
                        value={config.sensitivity}
                        onChange={(e) => onUpdate({ ...config, sensitivity: parseFloat(e.target.value) })}
                    />
                </label>

                <div className="module-footer-note">
                    Tightens transients and suppresses reverb tails.
                </div>
            </div>
        </BaseModule>
    );
};

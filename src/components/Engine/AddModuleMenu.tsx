// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import './AddModuleMenu.css';

interface AddModuleMenuProps {
  onAdd: (type: string) => void;
  onClose: () => void;
}

const MODULE_TYPES = [
  { label: 'Gain Control', type: 'Gain' },
  { label: 'Noise Expander', type: 'Expander' },
  { label: 'Noise Suppression', type: 'RNNoise' },
  { label: 'Audio Filter', type: 'Filter' },
  { label: 'Visualizer', type: 'Visualizer' },
];

export function AddModuleMenu({ onAdd, onClose }: AddModuleMenuProps) {
  return (
    <div className="add-module-menu">
      <div className="menu-header">
        <span>Add Module</span>
        <button className="close-menu-btn" onClick={onClose}>&times;</button>
      </div>
      <div className="menu-items">
        {MODULE_TYPES.map((item) => (
          <button
            key={item.type}
            className="menu-item"
            onClick={() => {
              onAdd(item.type);
              onClose();
            }}
          >
            {item.label}
          </button>
        ))}
      </div>
    </div>
  );
}

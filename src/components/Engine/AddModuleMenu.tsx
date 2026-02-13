// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import './AddModuleMenu.css';

interface AddModuleMenuProps {
  onAdd: (type: string) => void;
  onClose: () => void;
  existingTypes: string[];
}

const MODULE_TYPES = [
  { label: 'Gain Control', type: 'Gain' },
  { label: 'Noise Expander', type: 'Expander' },
  { label: 'Noise Suppression', type: 'RNNoise' },
  { label: 'Audio Filter', type: 'Filter' },
  { label: 'Visualizer', type: 'Visualizer' },
];

export function AddModuleMenu({ onAdd, onClose, existingTypes }: AddModuleMenuProps) {
  const filteredTypes = MODULE_TYPES.filter(item => !existingTypes.includes(item.type));

  return (
    <div className="add-module-menu">
      <div className="menu-header">
        <span>Add Module</span>
        <button className="close-menu-btn" onClick={onClose}>&times;</button>
      </div>
      <div className="menu-items">
        {filteredTypes.length > 0 ? (
          filteredTypes.map((item) => (
            <button
              key={item.type}
              className="menu-item"
              onClick={() => {
                onAdd(item.type);
              }}
            >
              {item.label}
            </button>
          ))
        ) : (
          <div className="no-modules-msg">All modules active</div>
        )}
      </div>
    </div>
  );
}

// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ADD MODULE MENU
// Modal for selecting and adding new audio modules to the chain.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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
  { label: 'Parametric EQ', type: 'ParametricEQ' },
  { label: 'Dynamic Compressor', type: 'Compressor' },
  { label: 'Vocal De-Esser', type: 'Deesser' },
  { label: 'Tube Saturation', type: 'Saturation' },
  { label: 'Brickwall Limiter', type: 'Limiter' },
  { label: 'Room De-Reverb', type: 'Dereverb' },
];

export function AddModuleMenu({ onAdd, onClose, existingTypes }: AddModuleMenuProps) {
  const filteredTypes = MODULE_TYPES.filter((item) => !existingTypes.includes(item.type)).sort(
    (a, b) => a.label.localeCompare(b.label),
  );

  return (
    <div className="add-module-menu">
      <div className="menu-items">
        {filteredTypes.length > 0 ? (
          filteredTypes.map((item) => (
            <button
              key={item.type}
              className="menu-item"
              onClick={() => {
                onAdd(item.type);
                // If this was the last item in the list, close the menu
                if (filteredTypes.indexOf(item) === filteredTypes.length - 1) {
                  onClose();
                }
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

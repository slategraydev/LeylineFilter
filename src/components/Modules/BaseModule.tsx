// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import React from 'react';
import './BaseModule.css';

interface BaseModuleProps {
  title: string;
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  children: React.ReactNode;
}

/**
 * # Component Wrapper
 * Provides consistent styling and the enable/disable toggle for all audio modules.
 */
export function BaseModule({ title, enabled, onToggle, children }: BaseModuleProps) {
  return (
    <div className={`module-card ${enabled ? 'active' : 'inactive'}`}>
      <div className="module-header">
        <div className="header-left">
          <h3>{title}</h3>
        </div>
        <label className="switch">
          <input
            type="checkbox"
            checked={enabled}
            onChange={(e) => onToggle(e.target.checked)}
          />
          <span className="slider round"></span>
        </label>
      </div>
      <div className="module-content">
        {children}
      </div>
    </div>
  );
}

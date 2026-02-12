// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

/**
 * # Grid System Constants
 * All dimensions are defined in 'Units'.
 * 1 Unit = GRID_SIZE (18px).
 */
export const GRID_UNIT_PX = 18;

export const SIDEBAR_WIDTH_PX = 340;

// Module dimensions in Units
export const MODULE_W_UNITS = 18; // 324px
export const MODULE_H_UNITS = 22; // Default 396px
export const GAP_UNITS = 1;       // 18px

// Specific heights for intelligent tight-fitting (calculated from content)
export const MODULE_HEIGHTS: Record<string, number> = {
  "Gain": 12,
  "Expander": 32,
  "RNNoise": 11,
  "Filter": 24,
  "Visualizer": 16,
  "default": 22
};

// Helper to convert units to pixels
export const toPx = (units: number) => units * GRID_UNIT_PX;

// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// LAYOUT UTILS
// Helpers for grid-based layout calculations.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

import { GridPosition } from '../types';
import { GAP_UNITS, MODULE_W_UNITS } from '../constants';

/**
 * # Layout Resolver
 * Finds the first available gap in the grid that can fit a module of height H.
 * This is used ONLY for initial placement or finding room.
 */
export const findFreeSlot = (
  moduleH: number,
  existingPositions: Record<string, GridPosition>,
  getH: (mid: string) => number,
  getW: (mid: string) => number,
  maxGx: number,
): GridPosition => {
  const existingIds = Object.keys(existingPositions);
  if (existingIds.length === 0) return { gx: 1, gy: 1 };

  // Brute force search for the first 1xH slot that doesn't overlap
  // We check row by row, column by column
  const colWidth = MODULE_W_UNITS + GAP_UNITS;
  const numCols = Math.max(1, Math.floor((maxGx + GAP_UNITS) / colWidth));

  // Search up to 500 rows deep
  for (let gy = 1; gy < 500; gy++) {
    for (let col = 0; col < numCols; col++) {
      const gx = 1 + col * colWidth;

      const hasOverlap = existingIds.some((id) => {
        const otherPos = existingPositions[id];
        const otherH = getH(id);
        const otherW = getW(id);

        const hOverlap =
          gx < otherPos.gx + otherW + GAP_UNITS && gx + MODULE_W_UNITS + GAP_UNITS > otherPos.gx;
        const vOverlap =
          gy < otherPos.gy + otherH + GAP_UNITS && gy + moduleH + GAP_UNITS > otherPos.gy;
        return hOverlap && vOverlap;
      });

      if (!hasOverlap) {
        return { gx, gy };
      }
    }
  }

  return { gx: 1, gy: 1 };
};

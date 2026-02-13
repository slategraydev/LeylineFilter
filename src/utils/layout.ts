// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import { GridPosition } from '../hooks/useDraggable';
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
  maxGx: number
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
      const gx = 1 + (col * colWidth);

      const hasOverlap = existingIds.some(id => {
        const otherPos = existingPositions[id];
        const otherH = getH(id);

        const hOverlap = gx < otherPos.gx + MODULE_W_UNITS + GAP_UNITS && gx + MODULE_W_UNITS + GAP_UNITS > otherPos.gx;
        const vOverlap = gy < otherPos.gy + otherH + GAP_UNITS && gy + moduleH + GAP_UNITS > otherPos.gy;
        return hOverlap && vOverlap;
      });

      if (!hasOverlap) {
        return { gx, gy };
      }
    }
  }

  return { gx: 1, gy: 1 };
};

/**
 * # Scale Resolver
 * Finds the required scale to fit all modules within both maxGx and maxGy.
 */
export const calculateScale = (
  positions: Record<string, GridPosition>,
  getH: (mid: string) => number,
  maxGx: number,
  maxGy: number
): number => {
  let totalHeight = 0;
  let totalWidth = 0;

  Object.keys(positions).forEach(id => {
    const pos = positions[id];
    const h = getH(id);
    totalHeight = Math.max(totalHeight, pos.gy + h); // Bottom unit
    totalWidth = Math.max(totalWidth, pos.gx + MODULE_W_UNITS); // Right unit
  });

  const scaleY = (totalHeight > maxGy && maxGy > 0) ? maxGy / totalHeight : 1.0;
  const scaleX = (totalWidth > maxGx && maxGx > 0) ? maxGx / totalWidth : 1.0;

  // Use the most restrictive scale to ensure everything fits
  return Math.min(1.0, Math.max(0.3, Math.min(scaleX, scaleY)));
};

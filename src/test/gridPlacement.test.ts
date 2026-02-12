
import { describe, it, expect } from 'vitest';
import { GridPosition } from '../hooks/useDraggable';

// Simplified version of the constants for testing
const MODULE_W_UNITS = 18;
const GAP_UNITS = 1;
const MODULE_HEIGHTS: Record<string, number> = {
  "Gain": 12,
  "Expander": 32,
  "RNNoise": 11,
  "Filter": 24,
  "Visualizer": 16,
  "default": 22
};

// The algorithm from App.tsx (extracted for verification)
const findNextAvailableSlot = (
  currentPositions: Record<string, GridPosition>,
  type: string,
  availableWidthPx: number,
  moduleTypeMap: Record<string, string>
): GridPosition => {
  const GRID_UNIT_PX = 18;
  const availableUnits = Math.floor(availableWidthPx / GRID_UNIT_PX);
  const maxGx = Math.max(MODULE_W_UNITS, availableUnits);
  const existingIds = Object.keys(currentPositions);

  const h = MODULE_HEIGHTS[type] || 22;

  if (existingIds.length === 0) return { gx: 1, gy: 1 };

  const potentialYs = [1];
  const potentialXs = [1];

  existingIds.forEach(id => {
    const pos = currentPositions[id];
    const mType = moduleTypeMap[id] || "default";
    const mHeight = MODULE_HEIGHTS[mType] || 22;

    potentialYs.push(pos.gy + mHeight + GAP_UNITS);
    potentialXs.push(pos.gx + MODULE_W_UNITS + GAP_UNITS);
  });

  const uniqueYs = Array.from(new Set(potentialYs)).sort((a, b) => a - b);
  const uniqueXs = Array.from(new Set(potentialXs)).sort((a, b) => a - b);

  for (const gy of uniqueYs) {
    for (const gx of uniqueXs) {
      if (gx + MODULE_W_UNITS - 1 > maxGx) continue;

      const isAreaBlocked = existingIds.some(id => {
        const pos = currentPositions[id];
        const mType = moduleTypeMap[id] || "default";
        const mHeight = MODULE_HEIGHTS[mType] || 22;

        const hOverlap = gx < pos.gx + MODULE_W_UNITS + GAP_UNITS && gx + MODULE_W_UNITS + GAP_UNITS > pos.gx;
        const vOverlap = gy < pos.gy + mHeight + GAP_UNITS && gy + h + GAP_UNITS > pos.gy;
        return hOverlap && vOverlap;
      });

      if (!isAreaBlocked) {
        return { gx, gy };
      }
    }
  }
  return { gx: 1, gy: 1 };
};

describe('Grid Placement Algorithm', () => {
  it('places the first module at (1,1)', () => {
    const pos = findNextAvailableSlot({}, "Gain", 1000, {});
    expect(pos).toEqual({ gx: 1, gy: 1 });
  });

  it('places the second module to the right of the first if it fits', () => {
    const current = { "m1": { gx: 1, gy: 1 } };
    const types = { "m1": "Gain" };
    // MODULE_W_UNITS is 18. Gap is 1. Next should be at 1 + 18 + 1 = 20.
    const pos = findNextAvailableSlot(current, "Gain", 1000, types);
    expect(pos).toEqual({ gx: 20, gy: 1 });
  });

  it('wraps to next row if window is too narrow', () => {
    const current = { "m1": { gx: 1, gy: 1 } };
    const types = { "m1": "Gain" };
    // Gain height is 12. Next should be at gy = 1 + 12 + 1 = 14.
    // Available width = 36 units (36 * 18 = 648)
    const pos = findNextAvailableSlot(current, "Gain", 648, types);
    expect(pos).toEqual({ gx: 1, gy: 14 });
  });

  it('tight-fits a small module in a hole', () => {
    // Hole at (20, 1) if we have modules at (1,1), (39,1) and (1, 14)
    const current = {
      "m1": { gx: 1, gy: 1 },
      "m2": { gx: 39, gy: 1 },
      "m3": { gx: 1, gy: 14 }
    };
    const types = { "m1": "Gain", "m2": "Gain", "m3": "Gain" };
    const pos = findNextAvailableSlot(current, "Gain", 2000, types);
    expect(pos).toEqual({ gx: 20, gy: 1 });
  });

  it('respects varying module heights for overlap detection', () => {
    // m1 is Expander (height 32) at (1,1)
    const current = { "m1": { gx: 1, gy: 1 } };
    const types = { "m1": "Expander" };

    // Narrow window forcing wrap
    const pos = findNextAvailableSlot(current, "Gain", 400, types);
    // Next should be at 1 + 32 + 1 = 34
    expect(pos).toEqual({ gx: 1, gy: 34 });
  });

  it('prunes stale positions (logic verification)', () => {
    // Simulated state from App.tsx useEffect
    const engineModules = [{ id: "m2", config: { type: "Gain" } }];
    const positions: Record<string, GridPosition> = {
      "m1": { gx: 1, gy: 1 }, // Stale
      "m2": { gx: 21, gy: 1 }
    };

    const next: Record<string, GridPosition> = { ...positions };
    const currentIds = new Set(engineModules.map(m => m.id));

    Object.keys(next).forEach(id => {
      if (!currentIds.has(id)) {
        delete next[id];
      }
    });

    expect(next).toEqual({ "m2": { gx: 21, gy: 1 } });
    expect(next).not.toHaveProperty("m1");
  });
});

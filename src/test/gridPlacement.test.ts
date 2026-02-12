import { describe, it, expect } from 'vitest';
import { GridPosition } from '../hooks/useDraggable';

const MODULE_W_UNITS = 18;
const GAP_UNITS = 1;

// The core displacement and compaction algorithm extracted for testing
export const resolvePositions = (
  id: string,
  pos: GridPosition,
  prev: Record<string, GridPosition>,
  getH: (mid: string) => number
): Record<string, GridPosition> => {
  // Start with a clean slate based on PREVIOUS positions to determine order
  const next = { ...prev, [id]: pos };

  // 1. Horizontal Displacement (Push Aside)
  if (pos.gx <= 5) {
    const shiftX = MODULE_W_UNITS + GAP_UNITS;
    Object.keys(prev).forEach(currId => {
      if (currId === id) return;
      const p = prev[currId];
      if (p.gx >= pos.gx) {
        next[currId] = { ...next[currId], gx: p.gx + shiftX };
      }
    });
  }

  // 2. Vertical Resolution
  // We process modules in their vertical order to ensure a stable "chain reaction"
  const ids = Object.keys(next).sort((a, b) => {
    const ya = next[a].gy;
    const yb = next[b].gy;
    if (ya !== yb) return ya - yb;
    // Ties: If one is the dragged module, it wins (comes first)
    if (a === id) return -1;
    if (b === id) return 1;
    return 0;
  });

  // Iteratively resolve overlaps in the sorted order
  for (let i = 0; i < ids.length; i++) {
    for (let j = i + 1; j < ids.length; j++) {
      const idA = ids[i];
      const idB = ids[j];

      const pA = next[idA];
      const pB = next[idB];
      const hA = getH(idA);
      const hB = getH(idB);

      const hOverlap = pA.gx < pB.gx + MODULE_W_UNITS + GAP_UNITS && pA.gx + MODULE_W_UNITS + GAP_UNITS > pB.gx;
      const vOverlap = pA.gy < pB.gy + hB + GAP_UNITS && pA.gy + hA + GAP_UNITS > pB.gy;

      if (hOverlap && vOverlap) {
        // A pushes B down (because A is "above" B in sorted order)
        next[idB] = { ...pB, gy: pA.gy + hA + GAP_UNITS };
        // We might have caused a new overlap with something else, so we need to re-sort or re-verify.
        // But since we are processing in order, it should propagate down.
      }
    }
  }
  // 3. Compaction Pass (Pull up while preserving order)
  const columns = Array.from(new Set(Object.values(next).map(p => p.gx)));
  columns.forEach(gx => {
    const colModules = Object.keys(next)
      .filter(mid => next[mid].gx === gx)
      .sort((a, b) => next[a].gy - next[b].gy);

    let currentY = 1;
    colModules.forEach(mid => {
      const p = next[mid];
      // Pull up to currentY if possible
      next[mid] = { ...p, gy: currentY };
      currentY = next[mid].gy + getH(mid) + GAP_UNITS;
    });
  });

  return next;
};
describe('Grid Displacement Logic', () => {
  const getH = () => 10; // Simple fixed height for tests

  it('pushes a module down when another is dropped on top', () => {
    const prev = { "m1": { gx: 10, gy: 1 } };
    const result = resolvePositions("m2", { gx: 10, gy: 1 }, prev, getH);

    expect(result["m2"]).toEqual({ gx: 10, gy: 1 });
    expect(result["m1"]).toEqual({ gx: 10, gy: 12 }); // 1 + 10 + 1
  });

  it('resolves chains of collisions', () => {
    const prev = {
      "m1": { gx: 10, gy: 1 },
      "m2": { gx: 10, gy: 12 }
    };
    const result = resolvePositions("m3", { gx: 10, gy: 1 }, prev, getH);

    expect(result["m3"]).toEqual({ gx: 10, gy: 1 });
    expect(result["m1"]).toEqual({ gx: 10, gy: 12 });
    expect(result["m2"]).toEqual({ gx: 10, gy: 23 }); // 12 + 10 + 1
  });

  it('compacts modules to remove gaps', () => {
    const prev = { "m1": { gx: 10, gy: 1 } };
    // Drop m2 at gy=50, it should be pulled up to gy=12
    const result = resolvePositions("m2", { gx: 10, gy: 50 }, prev, getH);

    expect(result["m1"]).toEqual({ gx: 10, gy: 1 });
    expect(result["m2"]).toEqual({ gx: 10, gy: 12 });
  });

  it('handles horizontal displacement at left edge', () => {
    const prev = {
      "m1": { gx: 1, gy: 1 },
      "m2": { gx: 1, gy: 12 }
    };
    // Drop m3 at gx=1 (left edge), should push m1 and m2 to the right
    const result = resolvePositions("m3", { gx: 1, gy: 1 }, prev, getH);

    expect(result["m3"]).toEqual({ gx: 1, gy: 1 });
    const shift = MODULE_W_UNITS + GAP_UNITS;
    expect(result["m1"].gx).toEqual(1 + shift);
    expect(result["m2"].gx).toEqual(1 + shift);
  });

  it('drops between modules in a column', () => {
    const prev = {
      "m1": { gx: 10, gy: 1 },
      "m2": { gx: 10, gy: 12 }
    };
    // Drop m3 at gy=5 (between 1 and 12).
    // m1(1) < m3(5) < m2(12).
    // Compaction: m1 at 1, m3 at 12, m2 at 23.
    const result = resolvePositions("m3", { gx: 10, gy: 5 }, prev, getH);

    expect(result["m1"].gy).toBeLessThan(result["m3"].gy);
    expect(result["m3"].gy).toBeLessThan(result["m2"].gy);

    // Exact positions after resolution + compaction:
    expect(result["m1"]).toEqual({ gx: 10, gy: 1 });
    expect(result["m3"]).toEqual({ gx: 10, gy: 12 });
    expect(result["m2"]).toEqual({ gx: 10, gy: 23 });
  });
});

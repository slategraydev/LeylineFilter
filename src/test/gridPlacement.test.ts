import { describe, it, expect } from 'vitest';
import { findFreeSlot, calculateScale } from '../utils/layout';

describe('Grid Layout Utilities', () => {
  const getH = () => 10;

  describe('findFreeSlot', () => {
    it('returns (1,1) for the first module', () => {
      const result = findFreeSlot(10, {}, getH, 40);
      expect(result).toEqual({ gx: 1, gy: 1 });
    });

    it('finds a slot next to an existing module', () => {
      const prev = { "m1": { gx: 1, gy: 1 } };
      // Module 1 is 18 wide + 1 gap = 19. Next slot should be at gx = 20
      const result = findFreeSlot(10, prev, getH, 40);
      expect(result).toEqual({ gx: 20, gy: 1 });
    });

    it('wraps to next row if no room in first row', () => {
      // Width 40 fits two modules (1 + 18 + 1 + 18 = 38).
      const prev = {
        "m1": { gx: 1, gy: 1 },
        "m2": { gx: 20, gy: 1 }
      };
      // Next module should wrap to row 12 (1 + 10 + 1)
      const result = findFreeSlot(10, prev, getH, 40);
      expect(result.gx).toBe(1);
      expect(result.gy).toBe(12);
    });
  });

  describe('calculateScale', () => {
    it('returns 1.0 if modules fit on screen', () => {
      const prev = { "m1": { gx: 1, gy: 1 } };
      const scale = calculateScale(prev, getH, 100, 100);
      expect(scale).toBe(1.0);
    });

    it('scales down if modules exceed maxGy', () => {
      const prev = { "m1": { gx: 1, gy: 100 } }; // gy 100 + height 10 = bottom 110 (100+10)
      // totalHeight = 110. maxGy = 50. scale = 50/110 = 0.4545...
      const scale = calculateScale(prev, getH, 100, 50);
      expect(scale).toBeLessThan(1.0);
      expect(scale).toBeCloseTo(50 / 110, 2);
    });

    it('scales down if modules exceed maxGx', () => {
      const prev = { "m1": { gx: 100, gy: 1 } }; // gx 100 + width 18 = 118
      // totalWidth = 118. maxGx = 50. scale = 50/118 = 0.4237...
      const scale = calculateScale(prev, getH, 50, 100);
      expect(scale).toBeLessThan(1.0);
      expect(scale).toBeCloseTo(50 / 118, 2);
    });
  });
});

import { describe, it, expect } from "vitest";
import { findFreeSlot } from "../utils/layout";

describe("Grid Layout Utilities", () => {
  const getH = () => 10;
  const getW = () => 18;

  describe("findFreeSlot", () => {
    it("returns (1,1) for the first module", () => {
      const result = findFreeSlot(10, {}, getH, getW, 40);
      expect(result).toEqual({ gx: 1, gy: 1 });
    });

    it("finds a slot next to an existing module", () => {
      const prev = { m1: { gx: 1, gy: 1 } };
      // maxGx 60 fits two standard columns (21 * 2 = 42)
      const result = findFreeSlot(10, prev, getH, getW, 60);
      expect(result).toEqual({ gx: 22, gy: 1 });
    });

    it("wraps to next row if no room in first row", () => {
      // With maxGx 40, only one standard column fits (colWidth 21)
      const prev = {
        m1: { gx: 1, gy: 1 },
      };
      // Next module should wrap to row 12 (1 + 10 + 1)
      const result = findFreeSlot(10, prev, getH, getW, 40);
      expect(result.gx).toBe(1);
      expect(result.gy).toBe(12);
    });
  });
});

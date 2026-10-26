import { describe, expect, it } from "vitest";
import { formatCommitTime } from "./commitTime";

// Local times, so the test works in any time zone.
const at = (y: number, m: number, d: number, h: number, min: number) => new Date(y, m - 1, d, h, min).getTime() / 1000;
const now = new Date(2026, 9, 26, 10, 0); // 26 Oct 2026, 10:00

describe("formatCommitTime", () => {
  it("says Today for any time since midnight", () => {
    expect(formatCommitTime(at(2026, 10, 26, 0, 5), now)).toMatch(/^Today, /);
    expect(formatCommitTime(at(2026, 10, 26, 9, 59), now)).toMatch(/^Today, /);
  });

  it("says Yesterday for the previous calendar day, even less than 24 hours ago", () => {
    expect(formatCommitTime(at(2026, 10, 25, 23, 50), now)).toMatch(/^Yesterday, /);
    expect(formatCommitTime(at(2026, 10, 25, 0, 0), now)).toMatch(/^Yesterday, /);
  });

  it("shows the full date for older commits", () => {
    const text = formatCommitTime(at(2026, 10, 24, 23, 59), now);
    expect(text).not.toMatch(/Today|Yesterday/);
    expect(text).toContain("2026");
  });

  it("works across a month and a year boundary", () => {
    expect(formatCommitTime(at(2026, 9, 30, 12, 0), new Date(2026, 9, 1, 8, 0))).toMatch(/^Yesterday, /);
    expect(formatCommitTime(at(2025, 12, 31, 12, 0), new Date(2026, 0, 1, 8, 0))).toMatch(/^Yesterday, /);
  });

  it("shows a future time (clock skew) with its date", () => {
    expect(formatCommitTime(at(2026, 10, 27, 9, 0), now)).toContain("2026");
  });
});

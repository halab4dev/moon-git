import { describe, expect, it } from "vitest";
import { displayUrl } from "./remoteUrl";

describe("displayUrl", () => {
  it("hides a password or token in the URL", () => {
    expect(displayUrl("https://me:ghp_secret@github.com/me/x.git")).toBe("https://me:***@github.com/me/x.git");
    expect(displayUrl("https://:token@host/x")).toBe("https://:***@host/x");
  });

  it("leaves other URLs as they are", () => {
    for (const url of [
      "https://github.com/me/x.git",
      "https://me@github.com/me/x.git",
      "ssh://git@host:2222/x.git",
      "git@github.com:me/x.git",
      "/Users/me/repos/x",
      "https://host/path:with:colons@at",
    ]) {
      expect(displayUrl(url)).toBe(url);
    }
  });
});

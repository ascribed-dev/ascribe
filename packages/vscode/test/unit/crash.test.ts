import { describe, expect, it } from "vitest";
import { CrashCounter } from "../../src/crash.js";

describe("CrashCounter", () => {
  it("allows restarts until the configured crash, which stops them", () => {
    const counter = new CrashCounter(3);
    expect(counter.recordCrash()).toBe(true);
    expect(counter.recordCrash()).toBe(true);
    expect(counter.recordCrash()).toBe(false);
    expect(counter.count).toBe(3);
  });

  it("stops at the first crash when the limit is 1", () => {
    expect(new CrashCounter(1).recordCrash()).toBe(false);
  });

  it("forgets crashes on reset", () => {
    const counter = new CrashCounter(2);
    counter.recordCrash();
    counter.reset();
    expect(counter.count).toBe(0);
    expect(counter.recordCrash()).toBe(true);
  });

  it("follows a changed limit", () => {
    const counter = new CrashCounter(2);
    counter.recordCrash();
    counter.setLimit(5);
    expect(counter.recordCrash()).toBe(true);
  });
});

// Runs the example and compares what it prints with the output the page
// shows beside it. Node runs TypeScript as it is, with its types stripped.
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import test from "node:test";
import assert from "node:assert/strict";

void test("greet.ts prints what greet.out.txt says", () => {
  const printed = execFileSync(process.execPath, ["examples/greet.ts"], { encoding: "utf8" });
  const expected = readFileSync("examples/greet.out.txt", "utf8");
  assert.equal(printed.replace(/\r\n/g, "\n"), expected.replace(/\r\n/g, "\n"));
});

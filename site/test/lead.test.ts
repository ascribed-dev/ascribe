// A docs page's lead: its quotes curled as the body's are, outside its code
// spans. It needs no build.
import { expect, test } from "vitest";
import { curlQuotes } from "../src/lead.ts";

test("an apostrophe is curled, and a quote opens after a space and closes after a word", () => {
  expect(curlQuotes(`The operation's address, "as written", and 'this'.`)).toBe(
    "The operation’s address, “as written”, and ‘this’.",
  );
  expect(curlQuotes(`"Quoted" at the start, and (“left”).`)).toBe(
    "“Quoted” at the start, and (“left”).",
  );
});

test("quotes in a code span, and in a tag's attributes, are left straight", () => {
  expect(
    curlQuotes(`Run <code>echo "it's"</code>, <a href="x">the page's</a> link, then it's done.`),
  ).toBe(`Run <code>echo "it's"</code>, <a href="x">the page’s</a> link, then it’s done.`);
});

test("quotes written as entities are curled, and other entities kept", () => {
  expect(curlQuotes("&quot;Tom &amp; Jerry&quot; &#39;s")).toBe("“Tom &amp; Jerry” ‘s");
});

test("a quote after a code span closes", () => {
  expect(curlQuotes("<code>ascribe.toml</code>'s file")).toBe("<code>ascribe.toml</code>’s file");
});

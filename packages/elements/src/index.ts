// @tessera/elements: registers Tessera's custom elements. Only
// <ascribe-tabs> has behavior; the rest are styled by the CSS file
// (`@tessera/elements/style.css`), which works without this script.

import { TesseraGroup } from "./group.js";
import { TesseraTab, TesseraTabs } from "./tabs.js";

export { TesseraGroup, TesseraTab, TesseraTabs };

const registry: [string, CustomElementConstructor][] = [
  ["ascribe-tabs", TesseraTabs],
  ["ascribe-tab", TesseraTab],
  ["ascribe-group", TesseraGroup],
];

for (const [name, constructor] of registry) {
  if (!customElements.get(name)) customElements.define(name, constructor);
}

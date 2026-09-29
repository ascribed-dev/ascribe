// @ascribed/elements: registers Ascribe's custom elements. Only
// <ascribe-tabs> has behavior; the rest are styled by the CSS file
// (`@ascribed/elements/style.css`), which works without this script.

import { AscribeGroup } from "./group.js";
import { AscribeTab, AscribeTabs } from "./tabs.js";

export { AscribeGroup, AscribeTab, AscribeTabs };

const registry: [string, CustomElementConstructor][] = [
  ["ascribe-tabs", AscribeTabs],
  ["ascribe-tab", AscribeTab],
  ["ascribe-group", AscribeGroup],
];

for (const [name, constructor] of registry) {
  if (!customElements.get(name)) customElements.define(name, constructor);
}

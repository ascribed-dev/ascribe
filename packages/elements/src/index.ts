// @ascribed/elements: registers Ascribe's custom elements, and adds a copy
// button to each code block. Only <ascribe-tabs> and the copy buttons have
// behavior; the rest are styled by the CSS file
// (`@ascribed/elements/style.css`), which works without this script.

import { startCopyButtons } from "./copy.js";
import { AscribeGroup } from "./group.js";
import { AscribeTab, AscribeTabs, choose } from "./tabs.js";
import { ELEMENT_GROUP, ELEMENT_TAB, ELEMENT_TABS } from "./names.js";

export { AscribeGroup, AscribeTab, AscribeTabs, choose };

const registry: [string, CustomElementConstructor][] = [
  [ELEMENT_TABS, AscribeTabs],
  [ELEMENT_TAB, AscribeTab],
  [ELEMENT_GROUP, AscribeGroup],
];

for (const [name, constructor] of registry) {
  if (!customElements.get(name)) customElements.define(name, constructor);
}

startCopyButtons();

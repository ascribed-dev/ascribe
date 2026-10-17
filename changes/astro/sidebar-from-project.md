- **Behavior change: `sidebar()` draws the project's navigation, and takes a site's own list as `nav`.** `sidebar(await getCollection("docs"), { entry })` gives a layout the project's sections and pages, in `ascribe.nav`'s order or the default one, from the build's `_ascribe/nav.json`, with no list for the site to keep. A site that passes its list moves it into the options, one line:

  ```diff
  - const groups = sidebar(nav, await getCollection("docs"), { entry });
  + const groups = sidebar(await getCollection("docs"), { nav, entry });
  ```

  A group gains `sections`, its nested sections, and `folder`, `page`, and `formatted` from the navigation; each page gains `label`, its title, and `formatted.label`, its HTML when the title has code. The starter's layout makes the move, and keeps its `src/nav.ts`. See [`sidebar()`](docs/content/astro/reference.md#sidebar).

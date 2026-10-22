- **Behavior change: the elements and review take Deep River and Shallows, Ascribe's new colors.** The element library, review's marks and comment threads, the review panel in Astro's dev toolbar, the HTML report, and an archive's frame move from the indigo palette to the mark's: deep blue-gray neutrals, river blue for links, notes, steps, and what changed, moss for a tip and what was added, and amber for focus. Important notes and moved blocks are violet, warnings yellow, and cautions and removed blocks crimson, kept apart from moss and river for readers with red-green color blindness. Every text color passes 4.5:1 on its background, in light and in dark. The HTML report's headings are set in the system's serif. A site that doesn't set the `--ascribe-*` and `--ascribe-review-*` custom properties looks different; their names are unchanged. To keep the previous look, add this to the site's stylesheet, after the element library's and review's:

  <details>
  <summary>The previous default colors</summary>

  ```css
  /* The element library */
  @supports (color: light-dark(#000, #fff)) {
    :root {
      --ascribe-border-color: light-dark(#e3e6ec, #393d46);
      --ascribe-muted-color: light-dark(#676c75, #a5a9b2);
      --ascribe-note-color: light-dark(#474dc6, #91a1fe);
      --ascribe-note-background: light-dark(#f5f6fe, #161b3a);
      --ascribe-tip-color: light-dark(#057256, #22c598);
      --ascribe-tip-background: light-dark(#e1fff2, #02251a);
      --ascribe-important-color: light-dark(#932f8e, #ffc8f9);
      --ascribe-important-background: light-dark(#fff2fd, #2d132b);
      --ascribe-warning-color: light-dark(#845502, #e0992e);
      --ascribe-warning-background: light-dark(#fef5eb, #2b1a03);
      --ascribe-caution-color: light-dark(#a73303, #fe7f57);
      --ascribe-caution-background: light-dark(#fef4f1, #341308);
      --ascribe-steps-color: light-dark(#5962e8, #91a1fe);
      --ascribe-steps-marker-text-color: light-dark(#ffffff, #0c0f16);
      --ascribe-tab-color: light-dark(#676c75, #a5a9b2);
      --ascribe-tab-active-color: light-dark(#5962e8, #91a1fe);
      --ascribe-tab-focus-color: light-dark(#5962e8, #b3c0ff);
      --ascribe-tab-hover-background: light-dark(#f7f9fd, #13171f);
      --ascribe-state-color: light-dark(#676c75, #a5a9b2);
      --ascribe-state-background: light-dark(#f0f3f7, #22272e);
      --ascribe-state-ga-color: light-dark(#057256, #22c598);
      --ascribe-state-ga-background: light-dark(#e1fff2, #02251a);
      --ascribe-state-preview-color: light-dark(#932f8e, #ffc8f9);
      --ascribe-state-preview-background: light-dark(#fff2fd, #2d132b);
      --ascribe-state-beta-color: light-dark(#474dc6, #91a1fe);
      --ascribe-state-beta-background: light-dark(#f5f6fe, #161b3a);
      --ascribe-state-deprecated-color: light-dark(#845502, #e0992e);
      --ascribe-state-deprecated-background: light-dark(#fef5eb, #2b1a03);
      --ascribe-state-removed-color: light-dark(#a73303, #fe7f57);
      --ascribe-state-removed-background: light-dark(#fef4f1, #341308);
    }
  }

  /* Review's marks and overlay */
  :root {
    --ascribe-review-added: #057256;
    --ascribe-review-added-background: #c7fae5;
    --ascribe-review-changed: #845502;
    --ascribe-review-changed-background: #fee9d0;
    --ascribe-review-removed: #ca3f02;
    --ascribe-review-removed-background: #fef4f1;
    --ascribe-review-moved: #932f8e;
    --ascribe-review-moved-background: #fff2fd;
    --ascribe-review-muted: #676c75;
    --ascribe-review-surface: #ffffff;
    --ascribe-review-flash: #fee9d0;
    --ascribe-review-focus: #5962e8;
    --ascribe-review-text: #22272e;
    --ascribe-review-border: #e3e6ec;
    --ascribe-review-thread: #f7f9fd;
    --ascribe-review-input: #ffffff;
    --ascribe-review-accent-text: #ffffff;
  }

  @media (prefers-color-scheme: dark) {
    :root:not([data-ascribe-scheme="light"]) {
      --ascribe-review-added: #22c598;
      --ascribe-review-added-background: #003929;
      --ascribe-review-changed: #e0992e;
      --ascribe-review-changed-background: #432900;
      --ascribe-review-removed: #fe7f57;
      --ascribe-review-removed-background: #511b0a;
      --ascribe-review-moved: #ffc8f9;
      --ascribe-review-moved-background: #461c43;
      --ascribe-review-muted: #a5a9b2;
      --ascribe-review-surface: #0c0f16;
      --ascribe-review-flash: #432900;
      --ascribe-review-focus: #b3c0ff;
      --ascribe-review-text: #f0f3f7;
      --ascribe-review-border: #393d46;
      --ascribe-review-thread: #13171f;
      --ascribe-review-input: #0c0f16;
      --ascribe-review-accent-text: #0c0f16;
    }
  }

  :root[data-ascribe-scheme="dark"] {
    --ascribe-review-added: #22c598;
    --ascribe-review-added-background: #003929;
    --ascribe-review-changed: #e0992e;
    --ascribe-review-changed-background: #432900;
    --ascribe-review-removed: #fe7f57;
    --ascribe-review-removed-background: #511b0a;
    --ascribe-review-moved: #ffc8f9;
    --ascribe-review-moved-background: #461c43;
    --ascribe-review-muted: #a5a9b2;
    --ascribe-review-surface: #0c0f16;
    --ascribe-review-flash: #432900;
    --ascribe-review-focus: #b3c0ff;
    --ascribe-review-text: #f0f3f7;
    --ascribe-review-border: #393d46;
    --ascribe-review-thread: #13171f;
    --ascribe-review-input: #0c0f16;
    --ascribe-review-accent-text: #0c0f16;
  }
  ```

  </details>

  See [the elements' custom properties](docs/content/astro/elements.md#custom-properties).

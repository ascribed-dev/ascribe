---
type: reference
title: Review in the site preview
description: "The Ascribe review app in astro dev's toolbar: its panel, what it compares, its comments, and the pages it has nothing to mark on."
available: ascribe 0.2.0
permalink: review-in-the-site-preview
---

In `astro dev`, Astro's dev toolbar has an **Ascribe review** app. It shows a pull request's changes and review comments on the real page, in your site's layout: the same [marks](how-it-works.md#the-marks) and threads as the editor's [page preview](../editor/review.md), from the same [`@ascribed/review`]({repo}/blob/main/packages/review/README.md) overlay. The integration's [`review` option](../astro/reference.md#options) can leave it out. [Review](index.md#review-in-the-site-preview) walks through reviewing a pull request with it; this page is the reference for the site's side.

## The panel

Nothing runs until you open the app: opening it starts review, and after **Stop Review**, the panel offers **Start Review**. The panel sits above the toolbar, in one row: the pull request and the base (**#12 against main**), your place in the changes ("3 of 10 on this page", click it for the breakdown), **Changes / As it will be / As it was**, next and previous change, **Comments**, **Refresh**, and **More review actions** (**⋯**), whose **Prompt agent** items copy a prompt for your agent about the page's changes or a fragment's pages ([In review](../agents/index.md#in-review)). Each thread's **Prompt agent** copies one about its comment. Past the last change it offers the next changed page, by its title. When `ascribe check` finds errors in the working tree, the panel [says how many](how-it-works.md#when-the-project-has-errors).

Close the panel to get the page back; it stays closed or open as you move between pages. With the panel closed, a dot on the toolbar button says you have comments you haven't submitted.

Review is on for the whole dev server, not one tab: once you start it, every page of the site that loads, in any tab or browser, shows the marks and threads without a click, until **Stop Review** or until `astro dev` stops.

Each mark's label and each thread's **Open source** opens the file at the line in your editor, through Vite's open-in-editor: set `LAUNCH_EDITOR` (for example, `LAUNCH_EDITOR=code`) to choose which.

## What it compares

Starting review compares the checkout with [the base](how-it-works.md#what-review-compares), and compares again after each rebuild, so the marks follow your edits on save. A [versioned](../versions/index.md) project is compared in every line, and a page shows the changes of the line it's in: `/3.4/install/` marks what changed in 3.4.

Comparing again after a save runs `ascribe diff` once, which takes about as long as an `ascribe build` of the project: under 50 ms on `examples/astro-site`. `astro dev` logs how long the first comparison took when review starts.

## Comments

Comments need the GitHub CLI, signed in, where `astro dev` runs ([Signing in to GitHub](how-it-works.md#signing-in-to-github)). Without it, or without a pull request, the app shows the changes only and says why. Comments, replies, and resolving work as in the page preview, and [map to the pull request](how-it-works.md#comments-and-the-pull-request) the same way: new comments are unsent until **Submit review…** in the panel sends them.

The page and the dev server talk over Vite's own connection, which only pages from the dev server can open. If your Vite config loosens that (`server.cors: true`, `server.allowedHosts: true`, or `legacy.skipWebSocketTokenCheck`), any web page open in your browser could use it, so the app keeps comments off and says which setting to change. It keeps them off too when the dev server listens on the network (`astro dev --host`, or a `server.host` other than localhost), where anyone who can reach the site could comment as you; the changes still show.

## Light and dark

The marks, the threads, and the panel are light or dark as the page is, not as your system is, so a light-only site stays readable when your system is in dark mode. The app reads the page's background (or, with none, its `color-scheme`) and sets `data-ascribe-scheme` on the root element; a site that sets `data-ascribe-scheme="light"` or `"dark"` there itself keeps its own.

## Pages with nothing to mark

- **A route that isn't an Ascribe page** (a changelog page of your own, an index) says so, and lists the pages the change touches, each a link.
- **A page whose layout drops the anchors**: the marks are placed by the `data-ascribe-source` attributes on each block, so a layout or component that rebuilds the content without them leaves nothing to mark. The panel says so, and lists the page's changes and threads instead, each with **Open source**.

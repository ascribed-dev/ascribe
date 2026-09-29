// The types of the `ascribe/preview` request (`crates/tessera-lsp/src/preview.rs`)
// and of the messages between the extension and the preview's webview.

/** The parameters of `ascribe/preview`. */
export interface PreviewParams {
  textDocument: { uri: string };
  /** A build name; without one the server uses the editor's build. */
  build?: string;
}

export interface PreviewBuild {
  name: string;
  /** Whether it's the editor's build: the picker's default. */
  editor: boolean;
  description: string;
}

export interface PreviewAsset {
  /** The reference as the page's HTML writes it, without `#fragment`. */
  reference: string;
  /** The source file, an absolute path. */
  path: string;
  kind: "image" | "link";
  /** Whether the preview may read the file: it is in the content root or an `assetRoots` directory. */
  servable: boolean;
}

export interface PreviewLink {
  href: string;
  path: string;
  id: string | null;
}

export interface PreviewSection {
  id: string;
  /** From 0. */
  line: number;
}

export interface PreviewPage {
  path: string;
  route: string;
  title: string | null;
  frontmatter: PageFrontmatter;
  html: string;
  assets: PreviewAsset[];
  links: PreviewLink[];
  sections: PreviewSection[];
}

/** The site output's frontmatter: `available` is a list of targets. */
export interface PageFrontmatter {
  available?: AvailabilityTarget[];
  [key: string]: unknown;
}

export interface AvailabilityTarget {
  target: string;
  dimension: string;
  states: string[];
  versions?: string[];
  text: string;
}

export interface PreviewProblem {
  severity: "error" | "warning" | "info";
  message: string;
}

export interface PreviewResult {
  build: string;
  builds: PreviewBuild[];
  projectRoot: string | null;
  contentRoot: string | null;
  /** Directories outside the content root that the page's assets are in. */
  assetRoots: string[];
  documentVersion: number | null;
  page: PreviewPage | null;
  problems: PreviewProblem[];
}

/** An asset the webview should show: the reference and the URL that reaches its source file. */
export interface WebviewAsset {
  reference: string;
  uri: string;
}

/** Extension to webview. */
export type ToWebview =
  | {
      type: "render";
      /** Counts renders; the webview echoes it. */
      seq: number;
      build: string;
      builds: PreviewBuild[];
      title: string | null;
      available: AvailabilityTarget[];
      html: string | null;
      assets: WebviewAsset[];
      problems: PreviewProblem[];
    }
  | { type: "reveal"; id: string };

/** Webview to extension. */
export type FromWebview =
  | { type: "ready" }
  | { type: "build"; name: string }
  | { type: "open"; href: string }
  | { type: "rendered"; seq: number; report: RenderReport }
  | { type: "images"; seq: number; images: ImageReport[] };

/** What the webview found in the page it just rendered, for tests and diagnostics. */
export interface RenderReport {
  headings: string[];
  elements: Record<string, number>;
  /** Whether the element library's custom elements are defined. */
  elementsDefined: boolean;
  /** Content security policy violations seen so far. */
  violations: string[];
}

export interface ImageReport {
  /** The `src` the page's image ended up with. */
  src: string;
  loaded: boolean;
  width: number;
}

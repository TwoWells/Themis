// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

// Deployed to GitHub Pages for the TwoWells/Themis repo. This is a *project*
// page (not the org root page), so the site is served from a `/Themis/`
// subpath — `site` is the full origin and `base` the subpath. Both must match
// the Pages URL or assets and internal links 404. (See `.github/workflows/docs.yml`.)
const SITE = "https://twowells.github.io";
const BASE = "/Themis/";

// Absolute URL of the social-preview image (`public/og.svg`). OpenGraph/Twitter
// image tags must be absolute, so this is built from `site` + `base` rather than
// a root-relative path. `BASE` already begins with `/`, so it resolves from the
// origin: https://twowells.github.io/Themis/og.svg.
const OG_IMAGE = new URL(`${BASE}og.svg`, SITE).href;

// https://astro.build/config
export default defineConfig({
  site: SITE,
  base: BASE,
  integrations: [
    starlight({
      title: "Themis",
      description: "A theme orchestrator CLI for Linux and macOS",
      social: [{ icon: "github", label: "GitHub", href: "https://github.com/TwoWells/Themis" }],
      // Default social-preview image for link unfurls. Starlight emits the
      // title/description OG tags itself; these add the image. Per-page
      // frontmatter can still override via the `head` field.
      head: [
        { tag: "meta", attrs: { property: "og:image", content: OG_IMAGE } },
        { tag: "meta", attrs: { name: "twitter:image", content: OG_IMAGE } },
        { tag: "meta", attrs: { name: "twitter:card", content: "summary_large_image" } },
      ],
      // "Edit" links resolve to the docs sources on GitHub. The Starlight
      // content collection (`website/src/content/docs`) is a symlink to the
      // repo's `docs/` tree, so the edit base points at `docs/` — Starlight
      // appends the page's path within the collection (e.g. `getting-started.md`).
      editLink: {
        baseUrl: "https://github.com/TwoWells/Themis/edit/main/docs/",
      },
      sidebar: [
        { label: "Home", slug: "index" },
        { label: "Getting Started", slug: "getting-started" },
        {
          label: "Guides",
          items: [
            { label: "Profiles & Palettes", slug: "guides/profiles" },
            { label: "Integration Types", slug: "guides/integrations" },
          ],
        },
        {
          label: "Reference",
          items: [
            { label: "CLI Commands", slug: "reference/cli" },
            { label: "Configuration", slug: "reference/config" },
            {
              label: "App Integrations",
              autogenerate: { directory: "reference/integrations" },
            },
          ],
        },
      ],
      customCss: [],
    }),
  ],
});

// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

// https://astro.build/config
export default defineConfig({
  integrations: [
    starlight({
      title: "Themis",
      description: "A theme orchestrator CLI for Linux",
      social: [{ icon: "github", label: "GitHub", href: "https://github.com/m-wells/themis" }],
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

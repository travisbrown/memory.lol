import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    // Single-page app: every URL falls back to index.html and is routed client-side,
    // matching the legacy CRA deployment under /app.
    adapter: adapter({ fallback: "index.html" }),
    paths: { base: "/app" },
  },
};

export default config;

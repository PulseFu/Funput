import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Builds the shared Funput desktop UI (Settings + Onboarding). The Tauri shells
// embed the `dist/` output via `frontendDist`.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  build: {
    outDir: "dist",
    emptyOutDir: true,
    target: "esnext",
  },
});

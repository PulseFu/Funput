import { mount } from "svelte";
import "./lib/tokens.css";
import App from "./App.svelte";
import { PLATFORM } from "./lib/api";

// Tag the root so platform-specific CSS (e.g. the Linux opaque background) applies
// on first paint. Set before mount to avoid a flash of the wrong style.
if (PLATFORM) document.documentElement.dataset.platform = PLATFORM;

const app = mount(App, { target: document.getElementById("app")! });

export default app;

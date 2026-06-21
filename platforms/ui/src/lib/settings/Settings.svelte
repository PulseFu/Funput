<script lang="ts">
  import { onMount } from "svelte";
  import * as api from "../api";
  import Sidebar from "../components/Sidebar.svelte";
  import InputMethod from "./panes/InputMethod.svelte";
  import Keyboard from "./panes/Keyboard.svelte";
  import Smart from "./panes/Smart.svelte";
  import General from "./panes/General.svelte";
  import Apps from "./panes/Apps.svelte";
  import About from "./panes/About.svelte";

  let settings = $state<api.Settings | null>(null);
  let active = $state("input");

  // "Chung" only holds launch-at-login, which is a no-op on Linux (fcitx5/desktop
  // own autostart there) — so hide that tab on Linux.
  const items = [
    { id: "input", label: "Kiểu gõ", icon: "⌨" },
    { id: "keyboard", label: "Phím chuyển", icon: "⌥" },
    { id: "smart", label: "Thông minh", icon: "✨" },
    { id: "apps", label: "Ứng dụng", icon: "🚫" },
    { id: "general", label: "Chung", icon: "⚙" },
    { id: "about", label: "Giới thiệu", icon: "ⓘ" },
  ].filter((it) => !(api.isLinux && it.id === "general"));

  onMount(async () => {
    settings = await api.getSettings();
  });
</script>

{#if settings}
  <div class="layout">
    <Sidebar {items} {active} onselect={(id) => (active = id)} />
    <main>
      {#if active === "input"}
        <InputMethod {settings} />
      {:else if active === "keyboard"}
        <Keyboard {settings} />
      {:else if active === "smart"}
        <Smart {settings} />
      {:else if active === "apps"}
        <Apps {settings} />
      {:else if active === "general"}
        <General {settings} />
      {:else}
        <About />
      {/if}
    </main>
  </div>
{/if}

<style>
  .layout {
    display: flex;
    height: 100vh;
  }
  main {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-xl);
  }
</style>

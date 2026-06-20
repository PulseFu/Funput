<script lang="ts">
  import { onMount } from "svelte";
  import Pane from "../Pane.svelte";
  import GlassCard from "../../components/GlassCard.svelte";
  import { getAppVersion, openUrl, LINKS } from "../../api";

  // Version comes from the app itself (Tauri reads it from tauri.conf.json, which CI
  // sets from the git tag) — single source of truth, no per-file drift.
  let version = $state("");
  onMount(async () => {
    version = await getAppVersion();
  });

  // Logo lives in platforms/ui/public/logo.png (served at /logo.png). Fall back to
  // the "FU" badge until the file is added.
  let hasLogo = $state(true);
</script>

<Pane title="Giới thiệu">
  <GlassCard>
    <div class="about">
      {#if hasLogo}
        <img class="logo" src="/logo.png" alt="Funput" onerror={() => (hasLogo = false)} />
      {:else}
        <div class="logo fallback">FU</div>
      {/if}
      <h2>Funput</h2>
      <p>Bộ gõ tiếng Việt — miễn phí, mã nguồn mở.</p>
      <p class="ver">Phiên bản {version}</p>
      <div class="links">
        <button type="button" onclick={() => openUrl(LINKS.github)}>GitHub</button>
        <button type="button" onclick={() => openUrl(LINKS.website)}>Website</button>
      </div>
    </div>
  </GlassCard>
</Pane>

<style>
  .about {
    text-align: center;
    padding: var(--space-md) 0;
  }
  .logo {
    width: 72px;
    height: 72px;
    margin: 0 auto var(--space-md);
    border-radius: 16px;
    display: block;
    object-fit: contain;
  }
  .logo.fallback {
    background: var(--accent);
    color: var(--accent-contrast);
    font-weight: 800;
    font-size: 28px;
    display: grid;
    place-items: center;
  }
  h2 {
    margin: 0 0 var(--space-xs);
    font-size: 18px;
  }
  p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
  }
  .ver {
    margin-top: var(--space-sm);
  }
  .links {
    display: flex;
    gap: var(--space-sm);
    justify-content: center;
    margin-top: var(--space-md);
  }
  .links button {
    appearance: none;
    border: 1px solid var(--control-bg, rgba(120, 120, 128, 0.32));
    background: var(--control-bg, rgba(120, 120, 128, 0.18));
    color: var(--text);
    font: inherit;
    font-size: 13px;
    padding: 6px 16px;
    border-radius: 999px;
    cursor: pointer;
    transition: filter 0.15s ease;
  }
  .links button:hover {
    filter: brightness(1.15);
  }
</style>

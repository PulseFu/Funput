<script lang="ts">
  import Pane from "../Pane.svelte";
  import GlassCard from "../../components/GlassCard.svelte";

  const version = "1.2026.1";

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
</style>

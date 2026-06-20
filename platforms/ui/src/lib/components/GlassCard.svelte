<script lang="ts">
  import type { Snippet } from "svelte";
  let { children }: { children: Snippet } = $props();
</script>

<div class="card">
  {@render children()}
</div>

<style>
  .card {
    padding: var(--space-lg);
    border-radius: var(--radius-card);
    background: var(--glass-bg);
    border: 1px solid var(--glass-border);
    backdrop-filter: blur(20px) saturate(180%);
    -webkit-backdrop-filter: blur(20px) saturate(180%);
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.25) inset;
  }
</style>

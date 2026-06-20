<script lang="ts">
  import type { Snippet } from "svelte";
  let { title, children }: { title: string; children: Snippet } = $props();
</script>

<section>
  <h1>{title}</h1>
  {@render children()}
</section>

<style>
  section {
    max-width: 460px;
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }
  h1 {
    font-size: 20px;
    font-weight: 700;
    margin: 0;
  }
</style>

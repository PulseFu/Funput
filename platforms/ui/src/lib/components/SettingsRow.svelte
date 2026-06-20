<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    title,
    subtitle = "",
    icon = "",
    control,
  }: { title: string; subtitle?: string; icon?: string; control: Snippet } = $props();
</script>

<div class="row">
  {#if icon}<span class="icon">{icon}</span>{/if}
  <div class="label">
    <div class="title">{title}</div>
    {#if subtitle}<div class="sub">{subtitle}</div>{/if}
  </div>
  <div class="control">{@render control()}</div>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-xs) 0;
  }
  .icon {
    width: 22px;
    text-align: center;
    color: var(--accent);
    font-size: 16px;
  }
  .label {
    min-width: 0;
  }
  .title {
    font-size: 14px;
  }
  .sub {
    font-size: 12px;
    color: var(--text-secondary);
    margin-top: 1px;
  }
  .control {
    margin-left: auto;
    flex-shrink: 0;
  }
  .row + :global(.row) {
    border-top: 1px solid var(--hairline);
  }
</style>

<script lang="ts">
  let { label }: { label: string } = $props();
</script>

<span class="cap">{label}</span>

<style>
  .cap {
    display: inline-block;
    padding: 3px 8px;
    border-radius: 6px;
    background: var(--glass-bg);
    border: 1px solid var(--glass-border);
    font-size: 12px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    box-shadow: 0 1px 1px var(--hairline);
  }
</style>

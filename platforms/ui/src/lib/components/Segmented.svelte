<script lang="ts" generics="T extends string">
  let {
    options,
    value,
    onchange,
  }: {
    options: { id: T; label: string }[];
    value: T;
    onchange?: (v: T) => void;
  } = $props();
</script>

<div class="seg" role="radiogroup">
  {#each options as opt (opt.id)}
    <button
      class:active={opt.id === value}
      role="radio"
      aria-checked={opt.id === value}
      onclick={() => onchange?.(opt.id)}
    >
      {opt.label}
    </button>
  {/each}
</div>

<style>
  .seg {
    display: inline-flex;
    background: var(--control-bg);
    border-radius: var(--radius-control);
    padding: 2px;
    gap: 2px;
  }
  .seg button {
    border: none;
    background: transparent;
    color: var(--text);
    padding: 4px 14px;
    border-radius: 8px;
    font-size: 13px;
    cursor: pointer;
    transition: background 0.15s ease;
  }
  .seg button.active {
    background: var(--glass-bg);
    box-shadow: 0 1px 2px var(--hairline);
    font-weight: 600;
  }
</style>

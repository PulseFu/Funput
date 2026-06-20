<script lang="ts">
  let {
    items,
    active,
    onselect,
  }: {
    items: { id: string; label: string; icon: string }[];
    active: string;
    onselect: (id: string) => void;
  } = $props();
</script>

<nav>
  <div class="brand">Funput</div>
  {#each items as it (it.id)}
    <button class:active={it.id === active} onclick={() => onselect(it.id)}>
      <span class="ic">{it.icon}</span>
      <span>{it.label}</span>
    </button>
  {/each}
</nav>

<style>
  nav {
    width: 200px;
    height: 100%;
    background: var(--sidebar-bg);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-right: 1px solid var(--hairline);
    padding: var(--space-md) var(--space-sm);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .brand {
    font-size: 13px;
    font-weight: 700;
    color: var(--text-secondary);
    padding: var(--space-sm) var(--space-sm) var(--space-md);
    -webkit-app-region: drag;
  }
  button {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    border: none;
    background: transparent;
    color: var(--text);
    text-align: left;
    padding: 7px var(--space-sm);
    border-radius: 8px;
    font-size: 13px;
    cursor: pointer;
  }
  button:hover {
    background: var(--control-bg);
  }
  button.active {
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .ic {
    width: 18px;
    text-align: center;
  }
</style>

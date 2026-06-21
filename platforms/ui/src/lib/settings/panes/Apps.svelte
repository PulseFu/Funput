<script lang="ts">
  import { onMount } from "svelte";
  import * as api from "../../api";
  import Pane from "../Pane.svelte";
  import GlassCard from "../../components/GlassCard.svelte";

  let { settings }: { settings: api.Settings } = $props();

  let recent = $state<api.ExcludedApp[]>([]);

  // Apps already excluded shouldn't show up in the "recent" add list.
  let addable = $derived(
    recent.filter((r) => !settings.excludedApps.some((e) => e.id === r.id)),
  );

  onMount(loadRecent);

  async function loadRecent() {
    recent = await api.listRecentApps();
  }

  function add(app: api.ExcludedApp) {
    settings.excludedApps = [...settings.excludedApps, app];
    api.addExcludedApp(app);
  }

  function remove(id: string) {
    settings.excludedApps = settings.excludedApps.filter((e) => e.id !== id);
    api.removeExcludedApp(id);
  }
</script>

<Pane title="Ứng dụng bỏ qua">
  <GlassCard>
    <div class="head">
      Mặc định tiếng Anh khi vào các app này — vẫn bật lại tiếng Việt bằng phím tắt.
    </div>

    {#if settings.excludedApps.length === 0}
      <div class="empty">Chưa có app nào. Thêm từ danh sách bên dưới.</div>
    {:else}
      {#each settings.excludedApps as app (app.id)}
        <div class="row">
          <div class="label">
            <div class="title">{app.name}</div>
            <div class="sub">{app.id}</div>
          </div>
          <button class="ghost" onclick={() => remove(app.id)} title="Bỏ khỏi danh sách">
            Xóa
          </button>
        </div>
      {/each}
    {/if}
  </GlassCard>

  <GlassCard>
    <div class="head">App gần đây</div>
    {#if addable.length === 0}
      <div class="empty">Chuyển qua lại các app một lúc để chúng hiện ở đây.</div>
    {:else}
      {#each addable as app (app.id)}
        <div class="row">
          <div class="label">
            <div class="title">{app.name}</div>
            <div class="sub">{app.id}</div>
          </div>
          <button class="add" onclick={() => add(app)}>Thêm</button>
        </div>
      {/each}
    {/if}
  </GlassCard>
</Pane>

<style>
  .head {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: var(--space-sm);
  }
  .empty {
    font-size: 13px;
    color: var(--text-secondary);
    padding: var(--space-xs) 0;
  }
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-xs) 0;
  }
  .row + .row {
    border-top: 1px solid var(--hairline);
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
    word-break: break-all;
  }
  button {
    margin-left: auto;
    flex-shrink: 0;
    border: none;
    border-radius: 7px;
    padding: 5px 12px;
    font-size: 13px;
    cursor: pointer;
  }
  .add {
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .ghost {
    background: var(--control-bg);
    color: var(--text-secondary);
  }
</style>

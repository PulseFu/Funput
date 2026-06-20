<script lang="ts">
  import * as api from "../../api";
  import Pane from "../Pane.svelte";
  import GlassCard from "../../components/GlassCard.svelte";
  import SettingsRow from "../../components/SettingsRow.svelte";
  import Toggle from "../../components/Toggle.svelte";

  let { settings }: { settings: api.Settings } = $props();

  function setSmart(on: boolean) {
    settings.smartRestore = on;
    api.setSmartRestore(on);
  }
  function setEager(on: boolean) {
    settings.eagerRestore = on;
    api.setEagerRestore(on);
  }
</script>

<Pane title="Gõ thông minh">
  <GlassCard>
    <SettingsRow
      title="Tự khôi phục tiếng Anh"
      subtitle="Từ không phải tiếng Việt giữ nguyên chữ gốc (card → card, không thành cảd)."
    >
      {#snippet control()}
        <Toggle checked={settings.smartRestore} onchange={setSmart} />
      {/snippet}
    </SettingsRow>
    <SettingsRow
      title="Khôi phục tức thì"
      subtitle="Đổi lại ngay khi từ không thể là tiếng Việt, không chờ dấu cách."
    >
      {#snippet control()}
        <Toggle
          checked={settings.eagerRestore}
          disabled={!settings.smartRestore}
          onchange={setEager}
        />
      {/snippet}
    </SettingsRow>
  </GlassCard>

  <GlassCard>
    <div class="ex-title">Ví dụ</div>
    <div class="ex"><code>text</code> → <code>text</code> (giữ tiếng Anh)</div>
    <div class="ex"><code>tieesng</code> → <code>tiếng</code> (gõ tiếng Việt)</div>
  </GlassCard>
</Pane>

<style>
  .ex-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: var(--space-sm);
  }
  .ex {
    font-size: 13px;
    padding: 3px 0;
  }
  code {
    font-family: ui-monospace, "SF Mono", "Cascadia Code", monospace;
    background: var(--control-bg);
    padding: 1px 6px;
    border-radius: 5px;
  }
</style>

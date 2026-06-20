<script lang="ts">
  import * as api from "../../api";
  import Pane from "../Pane.svelte";
  import GlassCard from "../../components/GlassCard.svelte";
  import SettingsRow from "../../components/SettingsRow.svelte";
  import Segmented from "../../components/Segmented.svelte";
  import KeyCap from "../../components/KeyCap.svelte";

  let { settings }: { settings: api.Settings } = $props();

  const options = api.HOTKEYS.map((h) => ({ id: h.id, label: h.caps.join(" ") }));
  let caps = $derived(api.HOTKEYS.find((h) => h.id === settings.toggleHotkey)?.caps ?? []);

  function pick(id: api.Hotkey) {
    settings.toggleHotkey = id;
    api.setToggleHotkey(id);
  }
</script>

<Pane title="Phím chuyển VI / EN">
  <GlassCard>
    <SettingsRow title="Phím tắt" subtitle="Bật/tắt gõ tiếng Việt nhanh.">
      {#snippet control()}
        <Segmented {options} value={settings.toggleHotkey} onchange={pick} />
      {/snippet}
    </SettingsRow>
    <SettingsRow title="Đang dùng">
      {#snippet control()}
        <span class="caps">
          {#each caps as c (c)}<KeyCap label={c} />{/each}
        </span>
      {/snippet}
    </SettingsRow>
  </GlassCard>
</Pane>

<style>
  .caps {
    display: inline-flex;
    gap: var(--space-xs);
  }
</style>

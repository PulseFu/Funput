<script lang="ts">
  import * as api from "../../api";
  import Pane from "../Pane.svelte";
  import GlassCard from "../../components/GlassCard.svelte";
  import SettingsRow from "../../components/SettingsRow.svelte";
  import Toggle from "../../components/Toggle.svelte";

  let { settings }: { settings: api.Settings } = $props();

  function setLaunch(on: boolean) {
    settings.launchAtLogin = on;
    api.setLaunchAtLogin(on);
  }
</script>

<Pane title="Chung">
  <GlassCard>
    <SettingsRow
      title="Khởi động cùng Windows"
      subtitle="Tự chạy Funput mỗi khi đăng nhập."
      icon="⏻"
    >
      {#snippet control()}
        <Toggle checked={settings.launchAtLogin} onchange={setLaunch} />
      {/snippet}
    </SettingsRow>
  </GlassCard>
</Pane>

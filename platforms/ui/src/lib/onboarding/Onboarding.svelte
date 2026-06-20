<script lang="ts">
  import { onMount } from "svelte";
  import * as api from "../api";
  import GlassCard from "../components/GlassCard.svelte";
  import Segmented from "../components/Segmented.svelte";
  import Toggle from "../components/Toggle.svelte";
  import KeyCap from "../components/KeyCap.svelte";

  let settings = $state<api.Settings | null>(null);
  let step = $state(0);
  const total = 4;

  onMount(async () => {
    settings = await api.getSettings();
  });

  async function next() {
    if (step < total - 1) {
      step += 1;
      return;
    }
    await api.completeOnboarding();
    await api.closeThisWindow();
  }
  function back() {
    if (step > 0) step -= 1;
  }
  function pickMethod(m: api.Method) {
    settings!.method = m;
    api.setMethod(m);
  }
  function setLaunch(on: boolean) {
    settings!.launchAtLogin = on;
    api.setLaunchAtLogin(on);
  }
</script>

{#if settings}
  <div class="wrap">
    <GlassCard>
      <div class="step">
        {#if step === 0}
          <div class="hero">👋</div>
          <h1>Chào mừng đến Funput</h1>
          <p>
            Gõ tiếng Việt ở mọi nơi trên {api.isLinux ? "Linux" : "Windows"} —
            miễn phí, mã nguồn mở.
          </p>
        {:else if step === 1}
          <div class="hero">⌨️</div>
          <h1>Chọn kiểu gõ</h1>
          <p>Có thể đổi bất cứ lúc nào trong Cài đặt.</p>
          <Segmented
            options={[
              { id: "telex", label: "Telex" },
              { id: "vni", label: "VNI" },
            ]}
            value={settings.method}
            onchange={pickMethod}
          />
        {:else if step === 2}
          <div class="hero">🔔</div>
          <h1>Cách hoạt động</h1>
          <p>
            Funput chạy nền ở khay hệ thống (icon “FU”). Nhấn
            <KeyCap label="Ctrl" /> <KeyCap label="`" /> để bật/tắt nhanh tiếng Việt.
          </p>
          {#if !api.isLinux}
            <p class="note">Lưu ý: không gõ được vào ứng dụng chạy quyền Administrator.</p>
          {/if}
        {:else}
          <div class="hero">✅</div>
          <h1>Sẵn sàng!</h1>
          {#if api.isLinux}
            <p>Bật Funput trong <strong>fcitx5-configtool</strong> rồi gõ thử ngay.</p>
          {:else}
            <p>Bật tự khởi động để Funput luôn có mặt khi bạn cần.</p>
            <label class="launch">
              <span>Khởi động cùng Windows</span>
              <Toggle checked={settings.launchAtLogin} onchange={setLaunch} />
            </label>
          {/if}
        {/if}
      </div>

      <div class="nav">
        {#if step > 0}
          <button class="ghost" onclick={back}>Quay lại</button>
        {/if}
        <div class="dots">
          {#each Array(total) as _, i (i)}
            <span class:on={i === step}></span>
          {/each}
        </div>
        <button class="primary" onclick={next}>
          {step < total - 1 ? "Tiếp tục" : "Bắt đầu"}
        </button>
      </div>
    </GlassCard>
  </div>
{/if}

<style>
  .wrap {
    height: 100vh;
    display: grid;
    place-items: center;
    padding: var(--space-xl);
  }
  .wrap :global(.card) {
    width: 100%;
    max-width: 420px;
  }
  .step {
    text-align: center;
    min-height: 220px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
  }
  .hero {
    font-size: 44px;
  }
  h1 {
    font-size: 20px;
    margin: 0;
  }
  p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 14px;
  }
  .note {
    font-size: 12px;
    opacity: 0.8;
  }
  .launch {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    margin-top: var(--space-sm);
  }
  .nav {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    margin-top: var(--space-lg);
  }
  .dots {
    display: flex;
    gap: 6px;
    margin: 0 auto;
  }
  .dots span {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--control-bg);
  }
  .dots span.on {
    background: var(--accent);
  }
  button {
    border: none;
    border-radius: var(--radius-control);
    padding: 8px 16px;
    font-size: 14px;
    cursor: pointer;
  }
  .primary {
    background: var(--accent);
    color: var(--accent-contrast);
    font-weight: 600;
  }
  .ghost {
    background: var(--control-bg);
    color: var(--text);
  }
</style>

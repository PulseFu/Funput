<script lang="ts">
  let {
    checked = false,
    disabled = false,
    onchange,
  }: { checked?: boolean; disabled?: boolean; onchange?: (v: boolean) => void } = $props();

  function toggle() {
    if (disabled) return;
    onchange?.(!checked);
  }
</script>

<button
  class="switch"
  class:on={checked}
  {disabled}
  role="switch"
  aria-checked={checked}
  aria-label="Bật/tắt"
  onclick={toggle}
>
  <span class="knob"></span>
</button>

<style>
  .switch {
    width: 40px;
    height: 24px;
    border-radius: var(--radius-pill);
    border: none;
    background: var(--control-bg);
    position: relative;
    cursor: pointer;
    transition: background 0.18s ease;
    padding: 0;
  }
  .switch.on {
    background: var(--accent);
  }
  .switch:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    transition: transform 0.18s ease;
  }
  .switch.on .knob {
    transform: translateX(16px);
  }
</style>

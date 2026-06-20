<script lang="ts">
  import Settings from "./lib/settings/Settings.svelte";
  import Onboarding from "./lib/onboarding/Onboarding.svelte";

  // The shell opens this page with ?view=settings or ?view=onboarding.
  const view = new URLSearchParams(location.search).get("view") ?? "settings";
</script>

{#if view === "onboarding"}
  <Onboarding />
{:else}
  <Settings />
{/if}

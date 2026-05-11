<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { windowState } from '$lib/stores/window';
  import Pill from '$lib/components/Pill.svelte';
  import CompactPanel from '$lib/components/CompactPanel.svelte';
  import ExpandedDashboard from '$lib/components/dashboard/ExpandedDashboard.svelte';

  let { children } = $props();

  let vaultLockTimer: ReturnType<typeof setTimeout>;

  function resetVaultTimer(): void {
    clearTimeout(vaultLockTimer);
    vaultLockTimer = setTimeout(async () => {
      const { lockVault } = await import('$lib/stores/vault');
      await lockVault();
    }, 5 * 60 * 1000);
  }

  async function startDrag(e: MouseEvent): Promise<void> {
    if (e.button !== 0) return;
    await getCurrentWindow().startDragging();
  }

  onMount(async () => {
    window.addEventListener('mousemove', resetVaultTimer);
    window.addEventListener('keydown', resetVaultTimer);
    resetVaultTimer();
  });
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="app" onmousedown={startDrag}>
  <Pill />

  {#if $windowState === 'panel'}
    <div class="panel-wrapper">
      <CompactPanel />
    </div>
  {:else if $windowState === 'expanded'}
    <div class="panel-wrapper">
      <ExpandedDashboard />
    </div>
  {/if}
</div>

<style>
  :global(html), :global(body) {
    margin: 0;
    overflow: hidden;
    font-family: -apple-system, 'Apple SD Gothic Neo', sans-serif;
    background: transparent;
  }

  .app {
    position: relative;
    width: 100%;
    height: 100vh;
    background: transparent;
    overflow: hidden;
  }

  .panel-wrapper {
    position: absolute;
    left: 62px;
    top: 50%;
    transform: translateY(-50%);
  }
</style>

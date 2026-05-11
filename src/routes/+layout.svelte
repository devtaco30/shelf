<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { goto } from '$app/navigation';
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
    await goto('/todo');
    window.addEventListener('mousemove', resetVaultTimer);
    window.addEventListener('keydown', resetVaultTimer);
    resetVaultTimer();
  });
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="app" onmousedown={startDrag}>
  <Pill />

  {#if $windowState === 'panel'}
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div onmousedown={(e) => e.stopPropagation()}>
      <CompactPanel>
        {@render children()}
      </CompactPanel>
    </div>
  {:else if $windowState === 'expanded'}
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div onmousedown={(e) => e.stopPropagation()}>
      <ExpandedDashboard />
    </div>
  {/if}
</div>

<style>
  :global(html), :global(body) {
    margin: 0;
    font-family: -apple-system, 'Apple SD Gothic Neo', sans-serif;
    background: transparent;
  }

  .app {
    display: flex;
    flex-direction: row;
    height: 100vh;
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0,0,0,0.2);
  }
</style>

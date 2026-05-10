<script lang="ts">
  import { onMount } from 'svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import { initWindowListener, collapsed } from '$lib/stores/window';
  import { goto } from '$app/navigation';

  let { children } = $props();

  let vaultLockTimer: ReturnType<typeof setTimeout>;

  function resetVaultTimer() {
    clearTimeout(vaultLockTimer);
    vaultLockTimer = setTimeout(async () => {
      const { lockVault } = await import('$lib/stores/vault');
      await lockVault();
    }, 5 * 60 * 1000); // 5분 비활성 시 자동 잠금
  }

  onMount(async () => {
    await initWindowListener();
    await goto('/todo');

    window.addEventListener('keydown', async (e) => {
      if (e.metaKey && e.shiftKey && e.key === 'S') {
        const { toggleCollapse } = await import('$lib/stores/window');
        await toggleCollapse();
      }
    });

    window.addEventListener('mousemove', resetVaultTimer);
    window.addEventListener('keydown', resetVaultTimer);
    resetVaultTimer();
  });
</script>

<div class="app" class:collapsed={$collapsed}>
  {#if !$collapsed}
    <main>{@render children()}</main>
  {/if}
  <Sidebar />
</div>

<style>
  :global(body) { margin: 0; font-family: -apple-system, sans-serif; background: transparent; }
  .app {
    display: flex;
    height: 100vh;
    background: #ffffff;
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0,0,0,0.2);
  }
  main { flex: 1; overflow: hidden; }
</style>

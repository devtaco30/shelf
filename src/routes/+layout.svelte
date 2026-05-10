<script lang="ts">
  import { onMount } from 'svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import { initWindowListener, collapsed } from '$lib/stores/window';
  import { goto } from '$app/navigation';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  let { children } = $props();

  let vaultLockTimer: ReturnType<typeof setTimeout>;

  function resetVaultTimer() {
    clearTimeout(vaultLockTimer);
    vaultLockTimer = setTimeout(async () => {
      const { lockVault } = await import('$lib/stores/vault');
      await lockVault();
    }, 5 * 60 * 1000);
  }

  async function closeWindow() {
    await getCurrentWindow().close();
  }

  async function minimizeWindow() {
    await getCurrentWindow().minimize();
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
  <!-- 타이틀바: 전체 너비에 걸친 드래그 영역 -->
  <div class="titlebar" data-tauri-drag-region>
    <div class="traffic-lights">
      <button class="dot close" on:click={closeWindow} title="닫기"></button>
      <button class="dot minimize" on:click={minimizeWindow} title="최소화"></button>
    </div>
  </div>

  <div class="body">
    {#if !$collapsed}
      <main>{@render children()}</main>
    {/if}
    <Sidebar />
  </div>
</div>

<style>
  :global(body) { margin: 0; font-family: -apple-system, sans-serif; background: transparent; }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #ffffff;
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0,0,0,0.2);
  }

  .titlebar {
    display: flex;
    align-items: center;
    height: 28px;
    background: #1e1e2e;
    border-radius: 12px 12px 0 0;
    flex-shrink: 0;
    padding: 0 10px;
    /* drag-region이 여기에 적용됨 */
  }

  .traffic-lights {
    display: flex;
    gap: 6px;
    /* 버튼 클릭은 드래그보다 우선하므로 별도 exclude 불필요 */
  }

  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: none;
    cursor: pointer;
    padding: 0;
  }

  .dot.close { background: #ff5f57; }
  .dot.minimize { background: #febc2e; }
  .dot:hover { filter: brightness(0.85); }

  .body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  main { flex: 1; overflow: hidden; }
</style>

<script lang="ts">
  import { onMount } from 'svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import { openTab, windowState } from '$lib/stores/window';
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

  async function startDrag(e: MouseEvent) {
    if (e.button !== 0) return;
    await getCurrentWindow().startDragging();
  }

  async function closeWindow() {
    await getCurrentWindow().close();
  }

  async function minimizeWindow() {
    await getCurrentWindow().minimize();
  }

  onMount(async () => {
    await goto('/todo');

    window.addEventListener('keydown', async (e) => {
      if (e.metaKey && e.shiftKey && e.key === 'S') {
        await openTab('todo');
      }
    });

    window.addEventListener('mousemove', resetVaultTimer);
    window.addEventListener('keydown', resetVaultTimer);
    resetVaultTimer();
  });
</script>

<div class="app">
  <!-- 타이틀바: mousedown으로 startDragging() 호출 -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="titlebar" on:mousedown={startDrag} role="presentation">
    <!-- 버튼 영역은 드래그 전파 차단 -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="traffic-lights" on:mousedown|stopPropagation>
      <button class="dot close" on:click={closeWindow} title="닫기"></button>
      <button class="dot minimize" on:click={minimizeWindow} title="최소화"></button>
    </div>
  </div>

  <div class="body">
    <main>{@render children()}</main>
    <Sidebar />
  </div>
</div>

<style>
  :global(html), :global(body) { margin: 0; font-family: -apple-system, sans-serif; background: transparent; }

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
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .dot.close { background: #ff5f57; }
  .dot.minimize { background: #febc2e; }

  .dot::after {
    content: '';
    position: absolute;
    font-size: 8px;
    font-weight: 900;
    color: rgba(0,0,0,0.45);
    opacity: 0;
    line-height: 1;
  }
  .dot.close::after { content: '✕'; }
  .dot.minimize::after { content: '−'; }
  .traffic-lights:hover .dot::after { opacity: 1; }

  .body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  main { flex: 1; overflow: hidden; min-width: 0; }
</style>

<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { goto } from '$app/navigation';
  import { windowState, activeTab, openTab, setState } from '$lib/stores/window';
  import type { Tab } from '$lib/stores/window';

  const tabs: { id: Tab; icon: string; label: string }[] = [
    { id: 'todo',  icon: '📋', label: '할 일' },
    { id: 'cal',   icon: '📅', label: '캘린더' },
    { id: 'vault', icon: '🔒', label: 'Vault' },
  ];

  let platform = 'macos';

  async function handleTabClick(tab: Tab) {
    await openTab(tab);
    if ($windowState === 'panel') await goto(`/${tab}`);
  }

  async function closeWindow()    { await getCurrentWindow().close(); }
  async function minimizeWindow() { await getCurrentWindow().minimize(); }

  onMount(async () => {
    platform = await invoke('get_platform');
  });
</script>

<aside class="pill">
  {#if platform === 'macos'}
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="traffic-lights" on:mousedown|stopPropagation>
      <button class="dot close"    on:click={closeWindow}    title="닫기"></button>
      <button class="dot minimize" on:click={minimizeWindow} title="최소화"></button>
    </div>
  {:else}
    <button class="win-close" on:click={closeWindow}>×</button>
  {/if}

  <div class="tabs">
    {#each tabs as tab}
      <button
        class="tab-btn"
        class:active={$windowState === 'panel' && $activeTab === tab.id}
        on:click={() => handleTabClick(tab.id)}
        title={tab.label}
      >
        {tab.icon}
        {#if $windowState === 'panel' && $activeTab === tab.id}
          <span class="indicator"></span>
        {/if}
      </button>
    {/each}
  </div>

  <div class="divider"></div>

  <button class="expand-btn" on:click={() => setState('expanded')} title="확장">⬜</button>
</aside>

<style>
  .pill {
    width: 52px;
    background: #1E1E2E;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 10px 0 12px;
    gap: 4px;
    flex-shrink: 0;
    height: 100vh;
  }

  .traffic-lights {
    display: flex;
    flex-direction: column;
    gap: 5px;
    margin-bottom: 8px;
    padding-top: 2px;
  }

  .dot {
    width: 10px; height: 10px;
    border-radius: 50%; border: none; cursor: pointer; padding: 0;
  }
  .dot.close    { background: #FF5F57; }
  .dot.minimize { background: #FEBC2E; }

  .win-close {
    background: none; border: none;
    color: rgba(255,255,255,0.6); cursor: pointer;
    font-size: 14px; margin-bottom: 8px;
  }

  .tabs {
    display: flex; flex-direction: column;
    align-items: center; gap: 4px; flex: 1;
  }

  .tab-btn {
    width: 36px; height: 36px;
    border: none; border-radius: 10px;
    background: transparent; cursor: pointer;
    font-size: 18px; display: flex; align-items: center; justify-content: center;
    position: relative; transition: background 0.15s;
  }
  .tab-btn:hover  { background: rgba(255,255,255,0.1); }
  .tab-btn.active { background: rgba(255,255,255,0.15); }

  .indicator {
    position: absolute; right: -8px; top: 50%; transform: translateY(-50%);
    width: 3px; height: 18px; background: #AAED3A; border-radius: 2px;
  }

  .divider {
    width: 28px; height: 0.5px;
    background: rgba(255,255,255,0.08); margin: 4px 0;
  }

  .expand-btn {
    width: 28px; height: 28px;
    border: none; border-radius: 7px;
    background: rgba(255,255,255,0.06); cursor: pointer;
    font-size: 13px; color: rgba(255,255,255,0.5);
    display: flex; align-items: center; justify-content: center;
    transition: background 0.15s;
  }
  .expand-btn:hover { background: rgba(255,255,255,0.12); color: #fff; }
</style>

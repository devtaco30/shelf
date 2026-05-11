<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { windowState, activeTab, openTab, setState } from '$lib/stores/window';
  import type { Tab } from '$lib/stores/window';

  const tabs: { id: Tab; icon: string; label: string }[] = [
    { id: 'todo',  icon: '📋', label: '할 일' },
    { id: 'cal',   icon: '📅', label: '캘린더' },
    { id: 'vault', icon: '🔒', label: 'Vault' },
  ];

  let platform = 'macos';

  async function handleTabClick(tab: Tab): Promise<void> {
    await openTab(tab);
  }

  async function handleFold(): Promise<void> {
    if ($windowState === 'pill') {
      await setState('panel');
    } else {
      await setState('pill');
    }
  }

  async function closeWindow():    Promise<void> { await getCurrentWindow().close(); }
  async function minimizeWindow(): Promise<void> { await getCurrentWindow().minimize(); }

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
        class:active={$windowState !== 'pill' && $activeTab === tab.id}
        on:click={() => handleTabClick(tab.id)}
        title={tab.label}
      >
        {tab.icon}
        {#if $windowState !== 'pill' && $activeTab === tab.id}
          <span class="indicator"></span>
        {/if}
      </button>
    {/each}
  </div>

  <div class="divider"></div>

  <button class="fold-ico" on:click={handleFold} title="패널 열기/닫기">
    <div class="fold-arrow" class:open={$windowState === 'pill'} class:closed={$windowState !== 'pill'}>
      <span></span>
      <span></span>
    </div>
  </button>
</aside>

<style>
  .pill {
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 52px;
    background: #1E1E2E;
    border-radius: 26px;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px 0;
    gap: 16px;
    border: 0.5px solid rgba(255,255,255,0.08);
  }

  .traffic-lights {
    display: flex;
    flex-direction: row;
    gap: 5px;
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
    font-size: 14px;
  }

  .tabs {
    display: flex; flex-direction: column;
    align-items: center; gap: 4px;
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
    background: rgba(255,255,255,0.08);
  }

  .fold-ico {
    width: 30px; height: 20px;
    background: rgba(255,255,255,0.06); border-radius: 6px;
    border: none; cursor: pointer;
    color: rgba(255,255,255,0.5);
    display: flex; align-items: center; justify-content: center;
    transition: background 0.15s;
  }
  .fold-ico:hover { background: rgba(255,255,255,0.12); color: rgba(255,255,255,0.8); }

  .fold-arrow {
    display: flex; flex-direction: column; gap: 3px;
    align-items: center; justify-content: center;
  }
  .fold-arrow span {
    display: block; width: 10px; height: 1.5px;
    background: currentColor; border-radius: 1px; transition: transform 0.2s;
  }

  /* panel/expanded open → ‹ pointing left */
  .fold-arrow.closed span:first-child { transform: rotate(35deg) translateY(1px); }
  .fold-arrow.closed span:last-child  { transform: rotate(-35deg) translateY(-1px); }
  /* pill only → › pointing right */
  .fold-arrow.open span:first-child { transform: rotate(-35deg) translateY(1px); }
  .fold-arrow.open span:last-child  { transform: rotate(35deg) translateY(-1px); }
</style>

<script lang="ts">
  import { activeTab, setState } from '$lib/stores/window';
  import type { Tab } from '$lib/stores/window';
  import { showAddTodoModal } from '$lib/stores/todos';
  import CompactTodo  from './compact/CompactTodo.svelte';
  import CompactCal   from './compact/CompactCal.svelte';
  import CompactVault from './compact/CompactVault.svelte';

  const TAB_TITLES: Record<Tab, string> = {
    todo: '📋 할 일', cal: '📅 캘린더', vault: '🔐 Vault',
  };

  async function handleExpand(): Promise<void> { await setState('expanded'); }
  async function handleClose(): Promise<void>  { await setState('pill'); }
</script>

<div class="compact-panel">
  <header class="cp-header">
    <span class="cp-title">{TAB_TITLES[$activeTab]}</span>
    <div class="cp-actions">
      {#if $activeTab === 'todo'}
        <button class="btn-add" on:mousedown|stopPropagation on:click={() => showAddTodoModal.set(true)}>＋</button>
      {/if}
      <button class="btn-expand" on:mousedown|stopPropagation on:click={handleExpand}>⬜ 확장</button>
      <button class="btn-close" on:mousedown|stopPropagation on:click={handleClose}>«</button>
    </div>
  </header>

  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="cp-body" on:mousedown|stopPropagation>
    {#if $activeTab === 'todo'}
      <CompactTodo />
    {:else if $activeTab === 'cal'}
      <CompactCal />
    {:else}
      <CompactVault />
    {/if}
  </div>
</div>

<style>
  .compact-panel {
    width: 280px;
    max-height: 460px;
    background: #fff;
    border-radius: 16px;
    display: flex; flex-direction: column; overflow: hidden;
  }

  .cp-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 11px 13px 10px; flex-shrink: 0;
  }

  .cp-title { font-size: 13px; font-weight: 600; }

  .cp-actions { display: flex; align-items: center; gap: 6px; }

  .btn-add {
    width: 22px; height: 22px; border-radius: 6px;
    background: #F0F0F0; border: none;
    font-size: 14px; line-height: 1;
    cursor: pointer; display: flex; align-items: center; justify-content: center;
    color: #666; transition: 0.15s;
  }
  .btn-add:hover { background: #E0E0E0; color: #111; }

  .btn-expand {
    padding: 5px 8px; border-radius: 6px;
    background: #AAED3A; border: none;
    font-size: 10px; font-weight: 600; cursor: pointer; color: #111;
  }
  .btn-expand:hover { background: #9bde2a; }

  .btn-close {
    background: none; border: none; cursor: pointer;
    color: #bbb; font-size: 13px; line-height: 1;
  }
  .btn-close:hover { color: #333; }

  .cp-body {
    overflow-y: auto; max-height: 400px;
  }
</style>

<script lang="ts">
  import { activeTab, setState } from '$lib/stores/window';

  const TAB_TITLES: Record<string, string> = {
    todo: '할 일', cal: '캘린더', vault: 'Vault',
  };

  async function handleExpand() { await setState('expanded'); }
  async function handleClose()  { await setState('pill'); }
</script>

<div class="compact-panel">
  <header class="cp-header">
    <span class="cp-title">{TAB_TITLES[$activeTab] ?? '할 일'}</span>
    <div class="cp-actions">
      <button class="btn-expand" on:click={handleExpand}>⬜ 확장</button>
      <button class="btn-close"  on:click={handleClose}>×</button>
    </div>
  </header>

  <div class="cp-body">
    <slot />
  </div>
</div>

<style>
  .compact-panel {
    width: 280px; height: 100vh;
    background: #fff;
    display: flex; flex-direction: column; overflow: hidden;
  }

  .cp-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 11px 13px 0; flex-shrink: 0;
  }

  .cp-title { font-size: 13px; font-weight: 600; }

  .cp-actions { display: flex; align-items: center; gap: 6px; }

  .btn-expand {
    padding: 3px 8px; border-radius: 6px;
    background: #AAED3A; border: none;
    font-size: 10px; font-weight: 600; cursor: pointer; color: #111;
  }

  .btn-close {
    background: none; border: none; cursor: pointer;
    color: #bbb; font-size: 15px; line-height: 1;
  }
  .btn-close:hover { color: #333; }

  .cp-body {
    flex: 1; overflow-y: auto; padding: 10px 13px 13px;
  }
</style>

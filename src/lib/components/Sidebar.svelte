<script lang="ts">
  import { activeTab, openTab, windowState } from '$lib/stores/window';
  import type { Tab } from '$lib/stores/window';

  const tabs: { id: Tab; icon: string; label: string }[] = [
    { id: 'todo', icon: '📋', label: 'Todo' },
    { id: 'cal', icon: '📅', label: 'Cal' },
    { id: 'vault', icon: '🔒', label: 'Vault' },
  ];
</script>

<aside class="sidebar">
  {#each tabs as tab}
    <button
      class="tab-btn"
      class:active={$activeTab === tab.id}
      onclick={() => openTab(tab.id)}
      title={tab.label}
    >
      {tab.icon}
    </button>
  {/each}

  <button
    class="tab-btn collapse-btn"
    onclick={() => openTab($activeTab)}
    title={$windowState === 'pill' ? '펼치기' : '접기'}
  >
    {$windowState === 'pill' ? '›' : '‹'}
  </button>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 48px;
    background: #1e1e2e;
    padding: 8px 0;
    gap: 4px;
    flex-shrink: 0;
  }
  .tab-btn {
    width: 36px; height: 36px;
    border: none; border-radius: 8px;
    background: transparent; cursor: pointer;
    font-size: 18px; display: flex; align-items: center; justify-content: center;
    color: white;
  }
  .tab-btn:hover { background: rgba(255,255,255,0.1); }
  .tab-btn.active { background: rgba(255,255,255,0.2); }

  .collapse-btn {
    margin-top: auto;
    font-size: 20px;
    color: rgba(255,255,255,0.4);
  }
  .collapse-btn:hover { color: rgba(255,255,255,0.8); }
</style>

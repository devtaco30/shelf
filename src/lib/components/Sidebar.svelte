<script lang="ts">
  import { activeTab, collapsed, toggleCollapse } from '$lib/stores/window';
  import type { Tab } from '$lib/stores/window';
  import { goto } from '$app/navigation';

  const tabs: { id: Tab; icon: string; label: string }[] = [
    { id: 'todo', icon: '📋', label: 'Todo' },
    { id: 'cal', icon: '📅', label: 'Cal' },
    { id: 'vault', icon: '🔒', label: 'Vault' },
  ];

  async function selectTab(tab: Tab) {
    if ($collapsed) await toggleCollapse();
    activeTab.set(tab);
    await goto(`/${tab}`);
  }
</script>

<aside class="sidebar" class:collapsed={$collapsed}>
  {#each tabs as tab}
    <button
      class="tab-btn"
      class:active={$activeTab === tab.id}
      on:click={() => selectTab(tab.id)}
      title={tab.label}
    >
      {tab.icon}
    </button>
  {/each}
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 48px;
    background: #1e1e2e;
    padding: 12px 0;
    gap: 4px;
    flex-shrink: 0;
  }
  .tab-btn {
    width: 36px; height: 36px;
    border: none; border-radius: 8px;
    background: transparent; cursor: pointer;
    font-size: 18px; display: flex; align-items: center; justify-content: center;
  }
  .tab-btn:hover { background: rgba(255,255,255,0.1); }
  .tab-btn.active { background: rgba(255,255,255,0.2); }
</style>

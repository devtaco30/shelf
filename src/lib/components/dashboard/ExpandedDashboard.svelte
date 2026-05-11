<script lang="ts">
  import { onMount } from 'svelte';
  import { loadTodos } from '$lib/stores/todos';
  import { loadEvents } from '$lib/stores/events';
  import { loadProjects } from '$lib/stores/projects';
  import { checkVaultState } from '$lib/stores/vault';
  import { activeTab } from '$lib/stores/window';
  import DashboardHeader  from './DashboardHeader.svelte';
  import ExpandedTodo     from './ExpandedTodo.svelte';
  import ExpandedCalendar from './ExpandedCalendar.svelte';
  import ExpandedVault    from './ExpandedVault.svelte';

  onMount(async () => {
    await Promise.all([loadTodos(), loadEvents(), loadProjects(), checkVaultState()]);
  });
</script>

<div class="expanded">
  <DashboardHeader />

  <div class="ex-content">
    {#if $activeTab === 'todo'}
      <ExpandedTodo />
    {:else if $activeTab === 'cal'}
      <ExpandedCalendar />
    {:else}
      <ExpandedVault />
    {/if}
  </div>
</div>

<style>
  .expanded {
    width: 560px; height: 100vh; background: #fff;
    display: flex; flex-direction: column; overflow: hidden;
  }
  .ex-content {
    flex: 1; overflow: hidden; display: flex; flex-direction: column;
  }
</style>

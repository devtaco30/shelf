<script lang="ts">
  import { onMount } from 'svelte';
  import { loadTodos } from '$lib/stores/todos';
  import { loadEvents } from '$lib/stores/events';
  import { loadProjects } from '$lib/stores/projects';
  import DashboardHeader from './DashboardHeader.svelte';
  import TodayTasks from './TodayTasks.svelte';
  import CalendarView from '$lib/components/cal/CalendarView.svelte';

  onMount(async () => {
    await Promise.all([loadTodos(), loadEvents(), loadProjects()]);
  });
</script>

<div class="expanded">
  <DashboardHeader />

  <div class="ex-body">
    <div class="ex-left">
      <CalendarView />
    </div>
    <div class="ex-right">
      <!-- TodaySchedule — Phase 2에서 구현 -->
      <p class="placeholder">오늘 일정<br><span>Phase 2에서 추가됩니다</span></p>
    </div>
  </div>

  <TodayTasks />
</div>

<style>
  .expanded {
    width: 560px; height: 100vh; background: #fff;
    display: flex; flex-direction: column; overflow: hidden;
  }
  .ex-body {
    display: grid; grid-template-columns: 1fr 200px;
    flex: 1; overflow: hidden; border-bottom: 0.5px solid #F0F0F0;
  }
  .ex-left { padding: 14px; border-right: 0.5px solid #F0F0F0; overflow-y: auto; }
  .ex-right { padding: 14px; overflow-y: auto; }
  .placeholder { font-size: 11px; color: #aaa; text-align: center; padding-top: 40px; line-height: 2; }
  .placeholder span { font-size: 10px; color: #ccc; }
</style>

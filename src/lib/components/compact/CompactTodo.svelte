<script lang="ts">
  import { onMount } from 'svelte';
  import { todos, loadTodos, toggleTodo, PRIORITY_COLORS } from '$lib/stores/todos';
  import { events, loadEvents } from '$lib/stores/events';
  import { projects, loadProjects } from '$lib/stores/projects';
  import { CATEGORY_COLORS } from '$lib/stores/projects';
  import AddTodoModal from '$lib/components/modals/AddTodoModal.svelte';

  const todayPrefix = new Date().toISOString().slice(0, 10);

  let showAddModal = false;

  $: todayTodos = $todos.filter(t => t.due_date?.startsWith(todayPrefix));
  $: doneTodayCount = todayTodos.filter(t => t.done).length;
  $: activeProjectCount = $projects.length;

  $: nextEventTime = (() => {
    const now = new Date();
    const upcoming = $events
      .filter(e => e.start_at.startsWith(todayPrefix))
      .map(e => new Date(e.start_at.replace(' ', 'T')))
      .filter(d => d >= now)
      .sort((a, b) => a.getTime() - b.getTime());
    if (!upcoming.length) return null;
    const h = String(upcoming[0].getHours()).padStart(2, '0');
    const m = String(upcoming[0].getMinutes()).padStart(2, '0');
    return `${h}:${m}`;
  })();

  function badgeStyle(category: string): string {
    const c = CATEGORY_COLORS[category] ?? { bg: '#F5F5F5', text: '#666' };
    return `background:${c.bg};color:${c.text}`;
  }

  onMount(async () => {
    await Promise.all([loadTodos(), loadEvents(), loadProjects()]);
  });
</script>

{#if showAddModal}
  <AddTodoModal on:close={() => (showAddModal = false)} />
{/if}

<div class="compact-todo">
  <!-- Stats row -->
  <div class="stats-row">
    <div class="stat-card">
      <span class="stat-num">{activeProjectCount}</span>
      <span class="stat-label">프로젝트</span>
    </div>
    <div class="stat-card">
      <span class="stat-num">{doneTodayCount}</span>
      <span class="stat-label">완료</span>
    </div>
    <div class="stat-card">
      <span class="stat-num">{nextEventTime ?? '없음'}</span>
      <span class="stat-label">다음 일정</span>
    </div>
  </div>

  <!-- Section header -->
  <div class="section-header">
    <span class="section-title">오늘 할 일</span>
    <button class="btn-add-task" on:click={() => (showAddModal = true)}>+</button>
  </div>

  <!-- Task list -->
  <ul class="task-list">
    {#each todayTodos as todo (todo.id)}
      {@const dotColor = PRIORITY_COLORS[todo.priority]}
      <li class="task-item">
        {#if dotColor}
          <span class="priority-dot" style="background:{dotColor}"></span>
        {:else}
          <span class="priority-dot empty"></span>
        {/if}
        <button
          class="checkbox" class:done={todo.done}
          on:click={() => toggleTodo(todo.id, !todo.done)}
          aria-label={todo.done ? '완료 취소' : '완료'}
        ></button>
        <span class="task-title" class:done={todo.done}>{todo.title}</span>
        <span class="badge" style={badgeStyle(todo.category)}>{todo.category}</span>
      </li>
    {:else}
      <li class="empty">오늘 할 일이 없어요</li>
    {/each}
  </ul>
</div>

<style>
  .compact-todo { display: flex; flex-direction: column; height: 100%; }

  .stats-row {
    display: grid; grid-template-columns: 1fr 1fr 1fr;
    gap: 6px; padding: 10px 12px; background: #FAFAFA;
    border-bottom: 0.5px solid #F0F0F0;
  }
  .stat-card {
    display: flex; flex-direction: column; align-items: center;
    padding: 6px 4px; background: #fff; border-radius: 8px;
    border: 0.5px solid #EFEFEF;
  }
  .stat-num   { font-size: 16px; font-weight: 700; line-height: 1; }
  .stat-label { font-size: 9px; color: #aaa; margin-top: 2px; }

  .section-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 12px 6px;
  }
  .section-title { font-size: 11px; font-weight: 600; color: #888; }
  .btn-add-task {
    width: 20px; height: 20px; border-radius: 50%;
    background: #AAED3A; border: none; font-size: 14px; line-height: 1;
    cursor: pointer; display: flex; align-items: center; justify-content: center;
    font-weight: 700; color: #111;
  }
  .btn-add-task:hover { background: #9bde2a; }

  .task-list { list-style: none; padding: 0 12px; margin: 0; flex: 1; overflow-y: auto; }
  .task-item {
    display: flex; align-items: center; gap: 5px;
    padding: 6px 0; border-bottom: 0.5px solid #F5F5F5;
  }
  .task-item:last-child { border: none; }

  .priority-dot {
    width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0;
  }
  .priority-dot.empty { background: transparent; }

  .checkbox {
    width: 14px; height: 14px; border-radius: 3px;
    border: 1.5px solid #ddd; background: transparent;
    cursor: pointer; flex-shrink: 0; padding: 0; transition: 0.15s;
  }
  .checkbox.done { background: #AAED3A; border-color: #AAED3A; }

  .task-title      { flex: 1; font-size: 12px; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .task-title.done { text-decoration: line-through; color: #bbb; }

  .badge {
    padding: 2px 6px; border-radius: 20px;
    font-size: 9px; font-weight: 600; white-space: nowrap; flex-shrink: 0;
  }

  .empty { font-size: 12px; color: #ccc; text-align: center; padding: 20px 0; }
</style>

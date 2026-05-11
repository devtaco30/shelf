<script lang="ts">
  import { onMount } from 'svelte';
  import { todos, loadTodos, toggleTodo, PRIORITY_COLORS, showAddTodoModal } from '$lib/stores/todos';
  import { events, loadEvents } from '$lib/stores/events';
  import { projects, loadProjects } from '$lib/stores/projects';
  import { CATEGORY_COLORS } from '$lib/stores/projects';
  import AddTodoModal from '$lib/components/modals/AddTodoModal.svelte';

  const todayPrefix = new Date().toISOString().slice(0, 10);

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

{#if $showAddTodoModal}
  <AddTodoModal on:close={() => showAddTodoModal.set(false)} />
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

  <div class="sec">오늘 할 일</div>

  <!-- Task list -->
  <ul style="list-style:none;padding:0 12px 13px;margin:0;line-height:1.2;">
    {#each todayTodos as todo, i (todo.id)}
      {@const dotColor = PRIORITY_COLORS[todo.priority]}
      <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
      <li style="display:flex;align-items:center;gap:6px;padding:8px 0;border-bottom:{i < todayTodos.length - 1 ? '0.5px solid #F0F0F0' : 'none'};">
        <span style="width:6px;height:6px;border-radius:50%;flex-shrink:0;background:{dotColor ?? 'transparent'};"></span>
        <div
          on:click={() => toggleTodo(todo.id, !todo.done)}
          style="width:14px;height:14px;min-width:14px;min-height:14px;border-radius:3px;border:1.5px solid {todo.done ? '#AAED3A' : '#ddd'};background:{todo.done ? '#AAED3A' : 'transparent'};cursor:pointer;flex-shrink:0;transition:0.15s;"
        ></div>
        <span style="flex:1;font-size:12px;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;{todo.done ? 'text-decoration:line-through;color:#bbb;' : ''}">{todo.title}</span>
        <span style="{badgeStyle(todo.category)};padding:2px 7px;border-radius:20px;font-size:9px;font-weight:600;white-space:nowrap;flex-shrink:0;">{todo.category}</span>
      </li>
    {:else}
      <li style="font-size:12px;color:#ccc;text-align:center;padding:20px 0;">오늘 할 일이 없어요</li>
    {/each}
  </ul>
</div>

<style>
  .compact-todo { display: flex; flex-direction: column; }

  .stats-row {
    display: flex; gap: 5px; padding: 10px 13px 0; margin-bottom: 0;
  }
  .stat-card {
    flex: 1; display: flex; flex-direction: column; align-items: center;
    padding: 6px 7px; background: #F7F7F7; border-radius: 7px;
  }
  .stat-num   { font-size: 16px; font-weight: 700; line-height: 1; }
  .stat-label { font-size: 9px; color: #aaa; margin-top: 2px; }

  .sec { font-size: 10px; font-weight: 600; color: #aaa; margin: 8px 0 5px; letter-spacing: 0.3px; }

  .task-list { list-style: none; padding: 0 12px; margin: 0; overflow-y: visible; }
  .task-item {
    display: flex; align-items: center; gap: 6px;
    padding: 5px 0; border-bottom: 0.5px solid #F0F0F0;
  }
  .task-item:last-child { border: none; }

  .priority-dot {
    width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0;
  }
  .priority-dot.empty { background: transparent; }

  .chk {
    width: 14px; height: 14px; border-radius: 3px;
    border: 1.5px solid #ddd; cursor: pointer; flex-shrink: 0; transition: 0.15s;
  }
  .chk.done { background: #AAED3A; border-color: #AAED3A; }

  .task-title      { flex: 1; font-size: 12px; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .task-title.done { text-decoration: line-through; color: #bbb; }

  .badge {
    padding: 2px 7px; border-radius: 20px;
    font-size: 9px; font-weight: 600; white-space: nowrap; flex-shrink: 0;
  }

  .empty { font-size: 12px; color: #ccc; text-align: center; padding: 20px 0; }
</style>

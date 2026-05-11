<script lang="ts">
  import { slide } from 'svelte/transition';
  import { onMount } from 'svelte';
  import { todos, loadTodos, toggleTodo, deleteTodo, PRIORITY_COLORS, showAddTodoModal, editingTodo } from '$lib/stores/todos';
  import type { Todo } from '$lib/stores/todos';
  import { showToast } from '$lib/stores/toast';
  import { events, loadEvents } from '$lib/stores/events';
  import { projects, loadProjects } from '$lib/stores/projects';
  import { CATEGORY_COLORS } from '$lib/stores/projects';

  const todayPrefix = new Date().toISOString().slice(0, 10);

  $: todayTodos = $todos.filter(t => t.due_date?.startsWith(todayPrefix));
  $: doneTodayCount = todayTodos.filter(t => t.done).length;
  $: activeProjectCount = $projects.length;

  let openId: number | null = null;

  let pendingDeleteId: number | null = null;

  $: displayTodos = todayTodos.filter(t => t.id !== pendingDeleteId);

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

  function toggleDetail(id: number): void {
    openId = openId === id ? null : id;
  }

  function handleEdit(e: MouseEvent, todo: Todo): void {
    e.stopPropagation();
    editingTodo.set(todo);
  }

  function handleDelete(e: MouseEvent, todo: Todo): void {
    e.stopPropagation();
    if (openId === todo.id) openId = null;
    // 이전 pending이 있으면 즉시 DB 삭제
    if (pendingDeleteId !== null) deleteTodo(pendingDeleteId);
    pendingDeleteId = todo.id;
    showToast(`"${todo.title}" 삭제됨`, () => {
      pendingDeleteId = null;
    }, 5000);
    // 5초 후 실제 삭제 (showToast 타이머와 동기화)
    setTimeout(async () => {
      if (pendingDeleteId === todo.id) {
        await deleteTodo(todo.id);
        pendingDeleteId = null;
      }
    }, 5000);
  }

  onMount(async () => {
    await Promise.all([loadTodos(), loadEvents(), loadProjects()]);
  });
</script>

<div class="compact-todo">
  <!-- Stats row -->
  <div style="display:flex;gap:5px;padding:0px 13px 0;">
    <div style="flex:1;background:#F7F7F7;border-radius:7px;padding:6px 7px;text-align:center;">
      <div style="font-size:16px;font-weight:700;line-height:1.25;">{activeProjectCount}</div>
      <div style="font-size:9px;color:#aaa;margin-top:1px;line-height:1.25;">프로젝트</div>
    </div>
    <div style="flex:1;background:#F7F7F7;border-radius:7px;padding:6px 7px;text-align:center;">
      <div style="font-size:16px;font-weight:700;line-height:1.25;">{doneTodayCount}</div>
      <div style="font-size:9px;color:#aaa;margin-top:1px;line-height:1.25;">완료</div>
    </div>
    <div style="flex:1;background:#F7F7F7;border-radius:7px;padding:6px 7px;text-align:center;">
      <div style="font-size:16px;font-weight:700;line-height:1.25;">{nextEventTime ?? '없음'}</div>
      <div style="font-size:9px;color:#aaa;margin-top:1px;line-height:1.25;">다음 일정</div>
    </div>
  </div>

  <div style="font-size:10px;font-weight:600;color:#aaa;margin:2px 13px 2px;letter-spacing:0.3px;">오늘 할 일</div>

  <!-- Task list -->
  <ul style="list-style:none;padding:0 12px 13px;margin:0;line-height:1.2;">
    {#each displayTodos as todo, i (todo.id)}
      {@const dotColor = PRIORITY_COLORS[todo.priority]}
      <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
      <li
        on:click={() => toggleDetail(todo.id)}
        style="cursor:pointer;border-bottom:{i < displayTodos.length - 1 || openId === todo.id ? '0.5px solid #F0F0F0' : 'none'};"
      >
        <!-- Row -->
        <div style="display:flex;align-items:center;gap:6px;padding:5px 0;">
          <span style="width:6px;height:6px;border-radius:50%;flex-shrink:0;background:{dotColor ?? 'transparent'};"></span>
          <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
          <div
            on:click|stopPropagation={() => toggleTodo(todo.id, !todo.done)}
            style="width:14px;height:14px;min-width:14px;min-height:14px;border-radius:3px;border:1.5px solid {todo.done ? '#AAED3A' : '#ddd'};background:{todo.done ? '#AAED3A' : 'transparent'};cursor:pointer;flex-shrink:0;transition:0.15s;"
          ></div>
          <span style="flex:1;font-size:12px;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;{todo.done ? 'text-decoration:line-through;color:#bbb;' : ''}">{todo.title}</span>
          <span style="{badgeStyle(todo.category)};padding:2px 7px;border-radius:20px;font-size:9px;font-weight:600;white-space:nowrap;flex-shrink:0;">{todo.category}</span>
        </div>
        <!-- Detail -->
        {#if openId === todo.id}
          <div transition:slide={{ duration: 150 }} style="padding:0 0 8px 20px;">
            {#if todo.due_date || todo.note}
              <div style="font-size:11px;color:#888;margin-bottom:7px;line-height:1.5;white-space:pre-wrap;">{[todo.due_date ? `마감: ${todo.due_date}` : '', todo.note].filter(Boolean).join('\n')}</div>
            {/if}
            <div style="display:flex;gap:5px;">
              <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
              <div
                on:click={(e) => handleEdit(e, todo)}
                style="padding:3px 10px;border-radius:6px;font-size:11px;font-weight:500;cursor:pointer;background:#F5F5F5;color:#555;"
              >✏️ 수정</div>
              <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
              <div
                on:click={(e) => handleDelete(e, todo)}
                style="padding:3px 10px;border-radius:6px;font-size:11px;font-weight:500;cursor:pointer;background:#FFF0F0;color:#EF4444;"
              >🗑 삭제</div>
            </div>
          </div>
        {/if}
      </li>
    {:else}
      <li style="font-size:12px;color:#ccc;text-align:center;padding:20px 0;">오늘 할 일이 없어요</li>
    {/each}
  </ul>
</div>

<style>
  .compact-todo { display: flex; flex-direction: column; }
</style>

<script lang="ts">
  import { todos, toggleTodo, createTodo, PRIORITY_COLORS } from '$lib/stores/todos';
  import { projects } from '$lib/stores/projects';
  import { CATEGORY_COLORS } from '$lib/stores/projects';
  import AddProjectModal from '$lib/components/modals/AddProjectModal.svelte';

  const CATEGORIES = ['작업', '클라이언트', '개인', 'work', '사일'];
  const todayPrefix = new Date().toISOString().slice(0, 10);

  let showAddProject = false;

  let selectedProjectId: number | null = null;

  let newTitle    = '';
  let newCategory = '작업';
  let newPriority = 0;

  $: filteredTodos = selectedProjectId === null
    ? $todos
    : $todos.filter(t => t.project_id === selectedProjectId);

  function taskCountForProject(pid: number): number {
    return $todos.filter(t => t.project_id === pid).length;
  }

  function badgeStyle(category: string): string {
    const c = CATEGORY_COLORS[category] ?? { bg: '#F5F5F5', text: '#666' };
    return `background:${c.bg};color:${c.text}`;
  }

  async function handleAddTodo(): Promise<void> {
    if (!newTitle.trim()) return;
    await createTodo(newTitle.trim(), '', todayPrefix, 'none', newCategory, newPriority);
    newTitle = '';
    newPriority = 0;
  }
</script>

{#if showAddProject}
  <AddProjectModal on:close={() => (showAddProject = false)} />
{/if}

<div class="expanded-todo">
  <!-- Left: Project panel -->
  <aside class="project-panel">
    <p class="panel-title">프로젝트</p>

    <button
      class="proj-item all-item"
      class:active={selectedProjectId === null}
      on:click={() => (selectedProjectId = null)}
    >
      <span class="proj-bar" style="background:#888"></span>
      <span class="proj-name">전체</span>
      <span class="task-count">{$todos.length}</span>
    </button>

    {#each $projects as proj (proj.id)}
      <button
        class="proj-item"
        class:active={selectedProjectId === proj.id}
        on:click={() => (selectedProjectId = proj.id)}
      >
        <span class="proj-bar" style="background:{proj.color}"></span>
        <div class="proj-info">
          <span class="proj-name">{proj.name}</span>
          <span class="proj-cat">{proj.category}</span>
        </div>
        <span class="task-count">{taskCountForProject(proj.id)}</span>
      </button>
    {/each}

    <button class="btn-new-project" on:click={() => (showAddProject = true)}>
      + 새 프로젝트
    </button>
  </aside>

  <!-- Right: Task panel -->
  <main class="task-panel">
    <ul class="task-list">
      {#each filteredTodos as todo (todo.id)}
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
          {#if todo.due_date}
            <span class="due-date">{todo.due_date.slice(5, 10)}</span>
          {/if}
        </li>
      {:else}
        <li class="empty">할 일이 없어요</li>
      {/each}
    </ul>

    <!-- Inline add row -->
    <div class="add-row">
      <div class="priority-btns">
        {#each [0, 1, 2, 3] as p}
          {@const color = PRIORITY_COLORS[p]}
          <button
            class="prio-btn" class:selected={newPriority === p}
            style={color ? `background:${color}` : 'background:#ddd'}
            on:click={() => (newPriority = p)}
            title={['없음','낮음','보통','높음'][p]}
          ></button>
        {/each}
      </div>
      <input
        class="add-input"
        bind:value={newTitle}
        placeholder="할 일 추가..."
        on:keydown={(e) => e.key === 'Enter' && handleAddTodo()}
      />
      <select class="cat-select" bind:value={newCategory}>
        {#each CATEGORIES as cat}<option value={cat}>{cat}</option>{/each}
      </select>
    </div>
  </main>
</div>

<style>
  .expanded-todo {
    display: flex; flex: 1; overflow: hidden;
  }

  /* Left project panel */
  .project-panel {
    width: 190px; flex-shrink: 0;
    border-right: 0.5px solid #F0F0F0;
    display: flex; flex-direction: column;
    padding: 12px 8px; overflow-y: auto;
    background: #FAFAFA;
  }
  .panel-title { font-size: 10px; font-weight: 600; color: #aaa; margin: 0 0 8px 4px; }

  .proj-item {
    display: flex; align-items: center; gap: 7px;
    padding: 7px 8px; border-radius: 8px;
    border: none; background: transparent; cursor: pointer;
    text-align: left; width: 100%; margin-bottom: 2px;
    transition: background 0.1s;
  }
  .proj-item:hover  { background: rgba(0,0,0,0.05); }
  .proj-item.active { background: rgba(0,0,0,0.08); }

  .proj-bar { width: 3px; height: 28px; border-radius: 2px; flex-shrink: 0; }
  .all-item .proj-bar { height: 14px; }

  .proj-info { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .proj-name { font-size: 12px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .proj-cat  { font-size: 9px; color: #aaa; }

  .task-count {
    font-size: 10px; color: #888;
    background: #eee; border-radius: 10px;
    padding: 1px 6px; flex-shrink: 0;
  }

  .btn-new-project {
    margin-top: 8px; padding: 7px 8px;
    border: 1px dashed #ddd; border-radius: 8px;
    background: none; cursor: pointer; color: #888;
    font-size: 11px; text-align: center; width: 100%;
    transition: border-color 0.15s, color 0.15s;
  }
  .btn-new-project:hover { border-color: #AAED3A; color: #111; }

  /* Right task panel */
  .task-panel {
    flex: 1; display: flex; flex-direction: column; overflow: hidden;
  }

  .task-list {
    flex: 1; list-style: none; padding: 8px 14px; margin: 0;
    overflow-y: auto;
  }
  .task-item {
    display: flex; align-items: center; gap: 6px;
    padding: 7px 0; border-bottom: 0.5px solid #F5F5F5;
  }
  .task-item:last-child { border: none; }

  .priority-dot {
    width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0;
  }
  .priority-dot.empty { background: transparent; }

  .checkbox {
    width: 15px; height: 15px; border-radius: 4px;
    border: 1.5px solid #ddd; background: transparent;
    cursor: pointer; flex-shrink: 0; padding: 0; transition: 0.15s;
  }
  .checkbox.done { background: #AAED3A; border-color: #AAED3A; }

  .task-title      { flex: 1; font-size: 13px; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .task-title.done { text-decoration: line-through; color: #bbb; }

  .badge {
    padding: 2px 7px; border-radius: 20px;
    font-size: 9px; font-weight: 600; white-space: nowrap; flex-shrink: 0;
  }
  .due-date { font-size: 10px; color: #aaa; flex-shrink: 0; }

  .empty { font-size: 12px; color: #ccc; text-align: center; padding: 30px 0; }

  /* Add row */
  .add-row {
    display: flex; align-items: center; gap: 6px;
    padding: 10px 14px; border-top: 0.5px solid #F0F0F0;
    flex-shrink: 0;
  }

  .priority-btns { display: flex; gap: 4px; }
  .prio-btn {
    width: 10px; height: 10px; border-radius: 50%;
    border: 1.5px solid transparent; cursor: pointer; padding: 0;
    transition: transform 0.1s;
  }
  .prio-btn.selected { border-color: #111; transform: scale(1.2); }

  .add-input {
    flex: 1; border: 0.5px solid #eee; border-radius: 6px;
    padding: 6px 8px; font-size: 12px; background: #F7F7F7; outline: none;
  }
  .add-input:focus { border-color: #AAED3A; background: #fff; }

  .cat-select {
    border: 0.5px solid #eee; border-radius: 6px;
    padding: 5px 4px; font-size: 11px; background: #F7F7F7;
    outline: none; max-width: 70px;
  }
</style>

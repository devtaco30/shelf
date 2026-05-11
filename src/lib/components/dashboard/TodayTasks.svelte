<script lang="ts">
  import { onMount } from 'svelte';
  import { todos, loadTodos, createTodo, toggleTodo, deleteTodo } from '$lib/stores/todos';
  import { CATEGORY_COLORS } from '$lib/stores/projects';

  const CATEGORIES = ['작업', '클라이언트', '개인', 'work', '사일'];
  const todayPrefix = new Date().toISOString().slice(0, 10);

  $: todayTodos = $todos.filter(t => t.due_date?.startsWith(todayPrefix));

  let newTitle    = '';
  let newCategory = '작업';

  async function handleAdd() {
    if (!newTitle.trim()) return;
    await createTodo(newTitle.trim(), '', todayPrefix, 'none', newCategory);
    newTitle = '';
  }

  function badgeStyle(category: string): string {
    const c = CATEGORY_COLORS[category] ?? { bg: '#F5F5F5', text: '#666' };
    return `background:${c.bg};color:${c.text}`;
  }

  onMount(loadTodos);
</script>

<section class="today-tasks">
  <p class="section-title">오늘 할 일</p>

  <ul class="task-list">
    {#each todayTodos as todo (todo.id)}
      <li class="task-item">
        <button
          class="checkbox" class:done={todo.done}
          on:click={() => toggleTodo(todo.id, !todo.done)}
          aria-label={todo.done ? '완료 취소' : '완료'}
        ></button>
        <span class="task-title" class:done={todo.done}>{todo.title}</span>
        <span class="badge" style={badgeStyle(todo.category)}>{todo.category}</span>
        <button class="btn-del" on:click={() => deleteTodo(todo.id)} aria-label="삭제">×</button>
      </li>
    {/each}
  </ul>

  <div class="task-input">
    <select bind:value={newCategory} class="cat-select">
      {#each CATEGORIES as cat}<option value={cat}>{cat}</option>{/each}
    </select>
    <input
      bind:value={newTitle}
      placeholder="할 일 추가..."
      on:keydown={(e) => e.key === 'Enter' && handleAdd()}
      class="title-input"
    />
    <button on:click={handleAdd} class="btn-add">추가</button>
  </div>
</section>

<style>
  .today-tasks { padding: 12px 14px; border-top: 0.5px solid #F0F0F0; }
  .section-title { font-size: 11px; font-weight: 600; color: #aaa; margin: 0 0 7px; }
  .task-list { list-style: none; padding: 0; margin: 0 0 8px; }
  .task-item {
    display: flex; align-items: center; gap: 6px;
    padding: 5px 0; border-bottom: 0.5px solid #F0F0F0;
  }
  .task-item:last-child { border: none; }
  .checkbox {
    width: 14px; height: 14px; border-radius: 3px;
    border: 1.5px solid #ddd; background: transparent;
    cursor: pointer; flex-shrink: 0; padding: 0; transition: 0.15s;
  }
  .checkbox.done { background: #AAED3A; border-color: #AAED3A; }
  .task-title      { flex: 1; font-size: 12px; }
  .task-title.done { text-decoration: line-through; color: #bbb; }
  .badge { padding: 2px 7px; border-radius: 20px; font-size: 9px; font-weight: 600; white-space: nowrap; }
  .btn-del { background: none; border: none; cursor: pointer; color: #ccc; font-size: 14px; line-height: 1; }
  .btn-del:hover { color: #333; }
  .task-input { display: flex; gap: 5px; }
  .cat-select {
    border: 0.5px solid #eee; border-radius: 6px;
    padding: 4px 6px; font-size: 11px; background: #F7F7F7; outline: none;
  }
  .title-input {
    flex: 1; border: 0.5px solid #eee; border-radius: 6px;
    padding: 5px 8px; font-size: 11px; background: #F7F7F7; outline: none;
  }
  .btn-add {
    padding: 5px 10px; border-radius: 6px;
    background: #111; color: #fff; border: none; font-size: 11px; cursor: pointer;
  }
  .btn-add:hover { background: #333; }
</style>

<script lang="ts">
  import type { Todo } from '$lib/stores/todos';
  import { toggleTodo, deleteTodo } from '$lib/stores/todos';

  export let todo: Todo;
</script>

<div class="todo-item" class:done={todo.done}>
  <input
    type="checkbox"
    checked={todo.done}
    on:change={() => toggleTodo(todo.id, !todo.done)}
  />
  <div class="content">
    <span class="title">{todo.title}</span>
    {#if todo.note}
      <span class="note">{todo.note}</span>
    {/if}
    {#if todo.recurrence !== 'none'}
      <span class="badge">{todo.recurrence}</span>
    {/if}
  </div>
  <button class="delete" on:click={() => deleteTodo(todo.id)}>×</button>
</div>

<style>
  .todo-item {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 8px 0;
    border-bottom: 1px solid #f0f0f0;
  }
  .done .title { text-decoration: line-through; opacity: 0.5; }
  .content { flex: 1; display: flex; flex-direction: column; gap: 2px; }
  .title { font-size: 14px; }
  .note { font-size: 12px; color: #888; }
  .badge { font-size: 10px; color: #666; background: #eee; padding: 1px 4px; border-radius: 4px; width: fit-content; }
  .delete { background: none; border: none; cursor: pointer; color: #ccc; font-size: 16px; padding: 0 4px; }
  .delete:hover { color: #f55; }
</style>

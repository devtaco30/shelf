<script lang="ts">
  import { onMount } from 'svelte';
  import { todos, loadTodos } from '$lib/stores/todos';
  import TodoItem from './TodoItem.svelte';
  import TodoForm from './TodoForm.svelte';

  onMount(loadTodos);
</script>

<div class="todo-list">
  <TodoForm />
  {#each $todos as todo (todo.id)}
    <TodoItem {todo} />
  {:else}
    <p class="empty">할 일이 없어요 🎉</p>
  {/each}
</div>

<style>
  .todo-list { padding: 12px; overflow-y: auto; height: 100%; }
  .empty { text-align: center; color: #aaa; margin-top: 40px; font-size: 13px; }
</style>

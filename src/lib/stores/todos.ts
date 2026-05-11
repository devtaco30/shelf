import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface Todo {
  id: number;
  title: string;
  note: string;
  done: boolean;
  due_date: string | null;
  recurrence: string;
  recurrence_next: string | null;
  project_id: number | null;
  category: string;
  created_at: string;
}

export const todos = writable<Todo[]>([]);

export async function loadTodos(): Promise<void> {
  const result = await invoke<Todo[]>('get_todos');
  todos.set(result);
}

export async function createTodo(
  title: string,
  note: string,
  due_date: string | null,
  recurrence: string,
  category: string = '작업',
): Promise<void> {
  await invoke('create_todo', { title, note, dueDate: due_date, recurrence, category });
  await loadTodos();
}

export async function updateTodo(
  id: number,
  title: string,
  note: string,
  due_date: string | null,
  category: string = '작업',
): Promise<void> {
  await invoke('update_todo', { id, title, note, dueDate: due_date, category });
  await loadTodos();
}

export async function toggleTodo(id: number, done: boolean): Promise<void> {
  await invoke('toggle_todo', { id, done });
  await loadTodos();
}

export async function deleteTodo(id: number): Promise<void> {
  await invoke('delete_todo', { id });
  await loadTodos();
}

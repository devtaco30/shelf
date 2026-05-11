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
  priority: number;
  created_at: string;
}

export const PRIORITY_COLORS: Record<number, string | null> = {
  0: null,
  1: '#4A9EFF',
  2: '#F59E0B',
  3: '#EF4444',
};

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
  priority: number = 0,
): Promise<void> {
  await invoke('create_todo', { title, note, dueDate: due_date, recurrence, category, priority });
  await loadTodos();
}

export async function updateTodo(
  id: number,
  title: string,
  note: string,
  due_date: string | null,
  category: string = '작업',
  priority: number = 0,
): Promise<void> {
  await invoke('update_todo', { id, title, note, dueDate: due_date, category, priority });
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

import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface CalEvent {
  id: number;
  title: string;
  start_at: string;
  end_at: string | null;
  recurrence: string;
  recurrence_next: string | null;
  todo_id: number | null;
  category: string;
  created_at: string;
}

export const events = writable<CalEvent[]>([]);

export async function loadEvents(): Promise<void> {
  const result = await invoke<CalEvent[]>('get_events');
  events.set(result);
}

export async function createEvent(
  title: string,
  start_at: string,
  end_at: string | null,
  recurrence: string,
  category: string = '작업',
): Promise<void> {
  await invoke('create_event', { title, startAt: start_at, endAt: end_at, recurrence, category });
  await loadEvents();
}

export async function deleteEvent(id: number): Promise<void> {
  await invoke('delete_event', { id });
  await loadEvents();
}

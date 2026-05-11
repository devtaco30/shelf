import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface Project {
  id: number;
  name: string;
  color: string;
  category: string;
  start_date: string | null;
  end_date: string | null;
  archived: boolean;
  created_at: string;
}

export const CATEGORY_COLORS: Record<string, { bg: string; text: string }> = {
  '작업':      { bg: '#E8F4FD', text: '#1A6FA8' },
  '클라이언트': { bg: '#FFF0F0', text: '#C0392B' },
  '개인':      { bg: '#F0FFF4', text: '#27AE60' },
  'work':      { bg: '#FFF8E1', text: '#E67E22' },
  '사일':      { bg: '#F3F0FF', text: '#6C5CE7' },
};

export const PROJECT_COLORS = [
  '#7C3AED', '#10B981', '#EF4444', '#F59E0B',
  '#06B6D4', '#EC4899', '#6B7280', '#4F46E5',
];

export const projects = writable<Project[]>([]);

export async function loadProjects(): Promise<void> {
  const result = await invoke<Project[]>('get_projects');
  projects.set(result);
}

export async function createProject(
  name: string,
  color: string,
  category: string,
  start_date: string | null,
  end_date: string | null,
): Promise<void> {
  await invoke('create_project', { name, color, category, startDate: start_date, endDate: end_date });
  await loadProjects();
}

export async function updateProject(
  id: number,
  name: string,
  color: string,
  category: string,
  start_date: string | null,
  end_date: string | null,
): Promise<void> {
  await invoke('update_project', { id, name, color, category, startDate: start_date, endDate: end_date });
  await loadProjects();
}

export async function deleteProject(id: number): Promise<void> {
  await invoke('delete_project', { id });
  await loadProjects();
}

export async function archiveProject(id: number): Promise<void> {
  await invoke('archive_project', { id });
  await loadProjects();
}

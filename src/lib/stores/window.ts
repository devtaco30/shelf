import { writable, get } from 'svelte/store';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';

export type WindowState = 'pill' | 'panel' | 'expanded';
export type Tab = 'todo' | 'cal' | 'vault';

const SIZES: Record<WindowState, { w: number; h: number }> = {
  pill:     { w: 52,  h: 250 },
  panel:    { w: 342, h: 480 },  // 52px pill + 10px gap + 280px panel
  expanded: { w: 622, h: 520 },  // 52px pill + 10px gap + 560px expanded
};

export const windowState = writable<WindowState>('pill');
export const activeTab   = writable<Tab>('todo');

export async function setState(next: WindowState): Promise<void> {
  const { w, h } = SIZES[next];
  await getCurrentWindow().setSize(new LogicalSize(w, h));
  windowState.set(next);
}

export async function openTab(tab: Tab): Promise<void> {
  const currentState = get(windowState);
  const currentTab   = get(activeTab);

  if (currentState === 'panel' && currentTab === tab) {
    await setState('pill');
    return;
  }

  activeTab.set(tab);
  await setState('panel');
}

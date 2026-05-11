import { writable, get } from 'svelte/store';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';

export type WindowState = 'pill' | 'panel' | 'expanded';
export type Tab = 'todo' | 'cal' | 'vault';

// 높이 고정 → pill이 top:50% 기준으로 항상 같은 위치 유지, 너비만 변경
const HEIGHT = 600;
const SIZES: Record<WindowState, { w: number; h: number }> = {
  pill:     { w: 52,  h: HEIGHT },
  panel:    { w: 342, h: HEIGHT },  // 52px pill + 10px gap + 280px panel
  expanded: { w: 622, h: HEIGHT },  // 52px pill + 10px gap + 560px expanded
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

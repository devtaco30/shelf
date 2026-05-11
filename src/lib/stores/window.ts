import { writable, get } from 'svelte/store';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';

export type WindowState = 'pill' | 'panel' | 'expanded';
export type Tab = 'todo' | 'cal' | 'vault';

const PILL_W     = 52;
const PANEL_W    = 332;  // 52 + 280
const EXPANDED_W = 612;  // 52 + 560
const HEIGHT     = 600;

export const windowState = writable<WindowState>('pill');
export const activeTab   = writable<Tab>('todo');

export async function setState(next: WindowState): Promise<void> {
  const widths: Record<WindowState, number> = {
    pill: PILL_W, panel: PANEL_W, expanded: EXPANDED_W,
  };
  await getCurrentWindow().setSize(new LogicalSize(widths[next], HEIGHT));
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

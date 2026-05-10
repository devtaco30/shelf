import { writable } from 'svelte/store';
import { getCurrentWindow, availableMonitors } from '@tauri-apps/api/window';

export type Tab = 'todo' | 'cal' | 'vault';

export const activeTab = writable<Tab>('todo');
export const collapsed = writable(false);

const EXPANDED_WIDTH = 360;
const COLLAPSED_WIDTH = 48;

export async function toggleCollapse(): Promise<void> {
  const win = getCurrentWindow();
  const isCollapsed = await new Promise<boolean>((resolve) => {
    const unsub = collapsed.subscribe((v) => { resolve(v); unsub(); });
  });

  if (isCollapsed) {
    await win.setSize({ type: 'Logical', width: EXPANDED_WIDTH, height: 600 });
    collapsed.set(false);
  } else {
    const { x, y } = await win.outerPosition();
    const monitors = await availableMonitors();
    const primary = monitors.find(m => m.isPrimary) ?? monitors[0];
    const screenW = primary.size.width / primary.scaleFactor;
    await win.setPosition({ type: 'Logical', x: screenW - COLLAPSED_WIDTH, y });
    await win.setSize({ type: 'Logical', width: COLLAPSED_WIDTH, height: 600 });
    collapsed.set(true);
  }
}

export async function initWindowListener(): Promise<void> {
  const win = getCurrentWindow();
  await win.onMoved(async ({ payload: { x } }) => {
    const monitors = await availableMonitors();
    const primary = monitors.find(m => m.isPrimary) ?? monitors[0];
    const screenW = primary.size.width / primary.scaleFactor;
    const size = await win.outerSize();
    const winW = size.width / primary.scaleFactor;

    if (x + winW >= screenW - 10) {
      await win.setSize({ type: 'Logical', width: COLLAPSED_WIDTH, height: 600 });
      await win.setPosition({ type: 'Logical', x: screenW - COLLAPSED_WIDTH, y: (await win.outerPosition()).y });
      collapsed.set(true);
    }
  });
}

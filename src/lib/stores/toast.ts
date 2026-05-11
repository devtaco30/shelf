import { writable } from 'svelte/store';

interface ToastState {
  msg: string;
  onUndo: (() => void) | null;
}

export const toast = writable<ToastState | null>(null);

let timer: ReturnType<typeof setTimeout> | null = null;

export function showToast(msg: string, onUndo: (() => void) | null = null, duration = 5000): void {
  if (timer) clearTimeout(timer);
  toast.set({ msg, onUndo });
  timer = setTimeout(() => {
    toast.set(null);
    timer = null;
  }, duration);
}

export function hideToast(): void {
  if (timer) clearTimeout(timer);
  timer = null;
  toast.set(null);
}

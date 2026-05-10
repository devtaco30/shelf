import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface VaultItem {
  id: number;
  title: string;
  created_at: string;
  updated_at: string;
}

export const vaultItems = writable<VaultItem[]>([]);
export const vaultUnlocked = writable(false);
export const vaultInitialized = writable(false);

export async function checkVaultState(): Promise<void> {
  const initialized = await invoke<boolean>('vault_initialized');
  const unlocked = await invoke<boolean>('is_vault_unlocked');
  vaultInitialized.set(initialized);
  vaultUnlocked.set(unlocked);
}

export async function setupVault(password: string): Promise<void> {
  await invoke('setup_vault', { password });
  vaultInitialized.set(true);
  vaultUnlocked.set(true);
}

export async function unlockVault(password?: string): Promise<void> {
  await invoke('unlock_vault', { password: password ?? null });
  vaultUnlocked.set(true);
  await loadVaultItems();
}

export async function lockVault(): Promise<void> {
  await invoke('lock_vault');
  vaultUnlocked.set(false);
  vaultItems.set([]);
}

export async function loadVaultItems(): Promise<void> {
  const items = await invoke<VaultItem[]>('get_vault_items');
  vaultItems.set(items);
}

export async function createVaultItem(title: string, content: string): Promise<void> {
  await invoke('create_vault_item', { title, content });
  await loadVaultItems();
}

export async function getVaultContent(id: number): Promise<string> {
  return await invoke<string>('get_vault_content', { id });
}

export async function deleteVaultItem(id: number): Promise<void> {
  await invoke('delete_vault_item', { id });
  await loadVaultItems();
}

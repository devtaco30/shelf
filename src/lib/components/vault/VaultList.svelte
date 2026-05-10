<script lang="ts">
  import { onMount } from 'svelte';
  import { vaultItems, loadVaultItems, getVaultContent, deleteVaultItem, lockVault } from '$lib/stores/vault';
  import VaultForm from './VaultForm.svelte';

  let revealed: Record<number, string> = {};

  async function reveal(id: number) {
    if (revealed[id]) {
      const copy = { ...revealed };
      delete copy[id];
      revealed = copy;
    } else {
      const content = await getVaultContent(id);
      revealed = { ...revealed, [id]: content };
    }
  }

  onMount(loadVaultItems);
</script>

<div class="vault-list">
  <div class="header">
    <span>🔒 Vault</span>
    <button class="lock-btn" on:click={lockVault}>잠금</button>
  </div>

  <VaultForm />

  {#each $vaultItems as item (item.id)}
    <div class="item">
      <div class="item-header">
        <span class="title">{item.title}</span>
        <div class="actions">
          <button on:click={() => reveal(item.id)}>
            {revealed[item.id] ? '숨기기' : '보기'}
          </button>
          <button class="delete" on:click={() => deleteVaultItem(item.id)}>×</button>
        </div>
      </div>
      {#if revealed[item.id]}
        <pre class="content">{revealed[item.id]}</pre>
      {/if}
    </div>
  {:else}
    <p class="empty">저장된 보안 메모가 없어요</p>
  {/each}
</div>

<style>
  .vault-list { padding: 12px; height: 100%; display: flex; flex-direction: column; gap: 8px; overflow-y: auto; }
  .header { display: flex; justify-content: space-between; align-items: center; font-weight: bold; }
  .lock-btn { font-size: 11px; padding: 4px 8px; border: 1px solid #ddd; border-radius: 4px; background: none; cursor: pointer; }
  .item { border: 1px solid #eee; border-radius: 8px; padding: 10px; }
  .item-header { display: flex; justify-content: space-between; align-items: center; }
  .title { font-size: 14px; font-weight: 500; }
  .actions { display: flex; gap: 4px; }
  .actions button { padding: 2px 8px; font-size: 11px; border: 1px solid #ddd; border-radius: 4px; background: none; cursor: pointer; }
  .delete:hover { border-color: #f55; color: #f55; }
  .content { margin: 8px 0 0; padding: 8px; background: #f9f9f9; border-radius: 4px; font-size: 12px; word-break: break-all; white-space: pre-wrap; }
  .empty { text-align: center; color: #aaa; font-size: 13px; margin-top: 40px; }
</style>

<script lang="ts">
  import { createVaultItem } from '$lib/stores/vault';

  let title = '';
  let content = '';
  let open = false;

  async function submit() {
    if (!title.trim() || !content.trim()) return;
    await createVaultItem(title.trim(), content.trim());
    title = ''; content = ''; open = false;
  }
</script>

{#if !open}
  <button class="add-btn" on:click={() => (open = true)}>+ 보안 메모 추가</button>
{:else}
  <form on:submit|preventDefault={submit} class="form">
    <input bind:value={title} placeholder="제목" autofocus />
    <textarea bind:value={content} placeholder="내용 (니모닉, 비밀번호 등)" rows="4"></textarea>
    <div class="actions">
      <button type="submit">저장</button>
      <button type="button" on:click={() => (open = false)}>취소</button>
    </div>
  </form>
{/if}

<style>
  .add-btn { width: 100%; padding: 8px; background: none; border: 1px dashed #ddd; border-radius: 6px; cursor: pointer; color: #888; }
  .form { display: flex; flex-direction: column; gap: 6px; }
  .form input, .form textarea { padding: 8px; border: 1px solid #ddd; border-radius: 6px; font-size: 13px; font-family: monospace; }
  .actions { display: flex; gap: 6px; }
  .actions button { flex: 1; padding: 8px; border: none; border-radius: 6px; cursor: pointer; }
  .actions button[type="submit"] { background: #4a9eff; color: white; }
</style>

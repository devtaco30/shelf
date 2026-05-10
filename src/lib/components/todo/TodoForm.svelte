<script lang="ts">
  import { createTodo } from '$lib/stores/todos';

  let title = '';
  let note = '';
  let due_date = '';
  let recurrence = 'none';
  let open = false;

  async function submit() {
    if (!title.trim()) return;
    await createTodo(title.trim(), note.trim(), due_date || null, recurrence);
    title = ''; note = ''; due_date = ''; recurrence = 'none'; open = false;
  }
</script>

{#if !open}
  <button class="add-btn" on:click={() => (open = true)}>+ 할 일 추가</button>
{:else}
  <form on:submit|preventDefault={submit} class="form">
    <input bind:value={title} placeholder="할 일" autofocus />
    <input bind:value={note} placeholder="메모 (선택)" />
    <input bind:value={due_date} type="datetime-local" />
    <select bind:value={recurrence}>
      <option value="none">반복 없음</option>
      <option value="daily">매일</option>
      <option value="weekly">매주</option>
      <option value="monthly">매월</option>
    </select>
    <div class="actions">
      <button type="submit">추가</button>
      <button type="button" on:click={() => (open = false)}>취소</button>
    </div>
  </form>
{/if}

<style>
  .add-btn { width: 100%; padding: 8px; background: none; border: 1px dashed #ddd; border-radius: 6px; cursor: pointer; color: #888; }
  .add-btn:hover { border-color: #aaa; color: #555; }
  .form { display: flex; flex-direction: column; gap: 6px; padding: 8px; background: #f9f9f9; border-radius: 8px; }
  .form input, .form select { padding: 6px; border: 1px solid #ddd; border-radius: 4px; font-size: 13px; }
  .actions { display: flex; gap: 6px; }
  .actions button { flex: 1; padding: 6px; border: none; border-radius: 4px; cursor: pointer; }
  .actions button[type="submit"] { background: #4a9eff; color: white; }
</style>

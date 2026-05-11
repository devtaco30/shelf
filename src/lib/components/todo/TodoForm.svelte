<script lang="ts">
  import { createTodo } from '$lib/stores/todos';
  import TimeInput from './TimeInput.svelte';
  import { CATEGORY_COLORS } from '$lib/stores/projects';
  const CATEGORIES = ['작업', '클라이언트', '개인', 'work', '사일'];

  let title = '';
  let note = '';
  let date = '';
  let time = '';
  let useTime = false;
  let recurrence = 'none';
  let category = '작업';
  let open = false;

  $: if (!date) { useTime = false; time = ''; }
  $: if (!useTime) time = '';

  function combineDatetime(): string | null {
    if (!date) return null;
    return useTime && time ? `${date}T${time}` : `${date}T00:00`;
  }

  async function submit() {
    if (!title.trim()) return;
    await createTodo(title.trim(), note.trim(), combineDatetime(), recurrence, category);
    title = ''; note = ''; date = ''; time = ''; useTime = false; recurrence = 'none'; category = '작업'; open = false;
  }
</script>

{#if !open}
  <button class="add-btn" on:click={() => (open = true)}>+ 할 일 추가</button>
{:else}
  <form on:submit|preventDefault={submit} class="form">
    <input bind:value={title} placeholder="할 일" autofocus />
    <textarea bind:value={note} placeholder="메모 (선택)" rows={2}></textarea>
    <div class="date-row">
      <input bind:value={date} type="date" />
      {#if date}
        <label class="time-toggle">
          <input type="checkbox" bind:checked={useTime} />
          시간
        </label>
      {/if}
    </div>
    {#if useTime && date}
      <TimeInput bind:value={time} />
    {/if}
    <select bind:value={recurrence}>
      <option value="none">반복 없음</option>
      <option value="daily">매일</option>
      <option value="weekly">매주</option>
      <option value="monthly">매월</option>
    </select>
    <select bind:value={category}>
      {#each CATEGORIES as cat}
        <option value={cat}>{cat}</option>
      {/each}
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
  .form input[type="text"], .form input:not([type]), .form select, .form textarea {
    padding: 6px; border: 1px solid #ddd; border-radius: 4px; font-size: 13px; font-family: inherit;
  }
  .form input[type="date"] { padding: 5px 6px; border: 1px solid #ddd; border-radius: 4px; font-size: 13px; }
  .form textarea { resize: none; line-height: 1.5; }
  .date-row { display: flex; align-items: center; gap: 8px; }
  .time-toggle { display: flex; align-items: center; gap: 4px; font-size: 13px; color: #555; cursor: pointer; white-space: nowrap; }
  .actions { display: flex; gap: 6px; }
  .actions button { flex: 1; padding: 6px; border: none; border-radius: 4px; cursor: pointer; }
  .actions button[type="submit"] { background: #4a9eff; color: white; }
</style>

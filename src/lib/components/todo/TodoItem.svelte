<script lang="ts">
  import type { Todo } from '$lib/stores/todos';
  import { toggleTodo, deleteTodo, updateTodo } from '$lib/stores/todos';
  import TimeInput from './TimeInput.svelte';

  export let todo: Todo;

  let expanded = false;
  let editing = false;
  let editTitle = '';
  let editNote = '';
  let editDate = '';
  let editTime = '';
  let useTime = false;

  function formatDate(raw: string | null): string {
    if (!raw) return '';
    const d = new Date(raw);
    const mm = String(d.getMonth() + 1).padStart(2, '0');
    const dd = String(d.getDate()).padStart(2, '0');
    const hh = String(d.getHours()).padStart(2, '0');
    const min = String(d.getMinutes()).padStart(2, '0');
    if (hh === '00' && min === '00') return `${mm}/${dd}`;
    return `${mm}/${dd} ${hh}:${min}`;
  }

  function toggleExpand() {
    expanded = !expanded;
    if (!expanded) editing = false;
  }

  function startEdit() {
    editTitle = todo.title;
    editNote = todo.note;
    if (todo.due_date) {
      const [d, t] = todo.due_date.split('T');
      editDate = d ?? '';
      const timeStr = t ? t.slice(0, 5) : '';
      editTime = timeStr;
      useTime = timeStr !== '' && timeStr !== '00:00';
    } else {
      editDate = '';
      editTime = '';
      useTime = false;
    }
    editing = true;
  }

  async function saveEdit() {
    if (!editTitle.trim()) return;
    const due_date = editDate
      ? (useTime && editTime ? `${editDate}T${editTime}` : `${editDate}T00:00`)
      : null;
    await updateTodo(todo.id, editTitle.trim(), editNote.trim(), due_date, todo.category, todo.priority);
    editing = false;
    expanded = false;
  }
</script>

<div class="todo-item" class:done={todo.done} class:expanded>
  <!-- 항상 보이는 행 -->
  <div class="row-header">
    <input
      type="checkbox"
      checked={todo.done}
      on:change={() => toggleTodo(todo.id, !todo.done)}
    />
    <button class="content-btn" on:click={toggleExpand}>
      <div class="row-main">
        <span class="title">{todo.title}</span>
        {#if todo.due_date}
          <span class="due">{formatDate(todo.due_date)}</span>
        {/if}
      </div>
      {#if !expanded && todo.note}
        <span class="note-preview">{todo.note}</span>
      {/if}
      {#if todo.recurrence !== 'none'}
        <span class="badge">{todo.recurrence}</span>
      {/if}
    </button>
    <button class="delete" on:click={() => deleteTodo(todo.id)}>×</button>
  </div>

  <!-- 펼쳐진 영역 -->
  {#if expanded}
    <div class="expand-section">
      {#if !editing}
        {#if todo.note}
          <p class="note-full">{todo.note}</p>
        {:else}
          <p class="note-empty">메모 없음</p>
        {/if}
        <div class="expand-actions">
          <button on:click={startEdit}>수정</button>
        </div>
      {:else}
        <input class="edit-input" bind:value={editTitle} placeholder="할 일" />
        <textarea class="edit-textarea" bind:value={editNote} placeholder="메모" rows={3}></textarea>
        <div class="date-row">
          <input type="date" bind:value={editDate} />
          {#if editDate}
            <label class="time-toggle">
              <input type="checkbox" bind:checked={useTime} />
              시간
            </label>
          {/if}
        </div>
        {#if useTime && editDate}
          <TimeInput bind:value={editTime} />
        {/if}
        <div class="expand-actions">
          <button class="save-btn" on:click={saveEdit}>저장</button>
          <button on:click={() => (editing = false)}>취소</button>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .todo-item {
    border-bottom: 1px solid #f0f0f0;
  }
  .row-header {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 8px 0;
  }
  .content-btn {
    flex: 1;
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .content-btn:hover .title { color: #4a9eff; }
  .row-main { display: flex; align-items: baseline; gap: 6px; width: 100%; }
  .title { font-size: 14px; flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .due { font-size: 11px; color: #4a9eff; white-space: nowrap; flex-shrink: 0; }
  .note-preview { font-size: 12px; color: #888; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .badge { font-size: 10px; color: #666; background: #eee; padding: 1px 4px; border-radius: 4px; width: fit-content; }
  .delete { background: none; border: none; cursor: pointer; color: #ccc; font-size: 16px; padding: 0 4px; flex-shrink: 0; }
  .delete:hover { color: #f55; }

  .done .title { text-decoration: line-through; opacity: 0.5; }

  .expand-section {
    padding: 8px 10px 12px 26px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: #fafafa;
    border-radius: 0 0 6px 6px;
  }
  .note-full {
    margin: 0;
    font-size: 13px;
    color: #555;
    white-space: pre-wrap;
    line-height: 1.5;
  }
  .note-empty { margin: 0; font-size: 12px; color: #bbb; font-style: italic; }
  .expand-actions { display: flex; gap: 6px; }
  .expand-actions button {
    padding: 4px 12px;
    border: 1px solid #ddd;
    border-radius: 4px;
    background: white;
    cursor: pointer;
    font-size: 12px;
  }
  .expand-actions .save-btn { background: #4a9eff; color: white; border-color: #4a9eff; }

  .edit-input, .edit-textarea {
    width: 100%;
    padding: 5px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 13px;
    font-family: inherit;
    box-sizing: border-box;
  }
  .edit-textarea { resize: none; line-height: 1.5; }
  .date-row { display: flex; align-items: center; gap: 8px; }
  .date-row input[type="date"] { flex: 1; padding: 4px; border: 1px solid #ddd; border-radius: 4px; font-size: 12px; }
  .time-toggle { display: flex; align-items: center; gap: 4px; font-size: 12px; color: #555; cursor: pointer; white-space: nowrap; }
</style>

<script lang="ts">
  import { onMount } from 'svelte';
  import { events, loadEvents, createEvent, deleteEvent } from '$lib/stores/events';
  import { todos, loadTodos } from '$lib/stores/todos';

  let currentDate = new Date();
  let showForm = false;
  let newTitle = '';
  let newStartAt = '';

  $: year = currentDate.getFullYear();
  $: month = currentDate.getMonth();
  $: firstDay = new Date(year, month, 1).getDay();
  $: daysInMonth = new Date(year, month + 1, 0).getDate();
  $: days = Array.from({ length: daysInMonth }, (_, i) => i + 1);

  const MONTH_NAMES = ['1월','2월','3월','4월','5월','6월','7월','8월','9월','10월','11월','12월'];

  function datePrefix(day: number): string {
    return `${year}-${String(month + 1).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
  }

  function eventsOnDay(day: number): typeof $events {
    return $events.filter(e => e.start_at.startsWith(datePrefix(day)));
  }

  function todosOnDay(day: number): typeof $todos {
    return $todos.filter(t => t.due_date?.startsWith(datePrefix(day)));
  }

  function prevMonth() { currentDate = new Date(year, month - 1, 1); }
  function nextMonth() { currentDate = new Date(year, month + 1, 1); }

  async function submit() {
    if (!newTitle.trim() || !newStartAt) return;
    await createEvent(newTitle.trim(), newStartAt, null, 'none');
    newTitle = ''; newStartAt = ''; showForm = false;
  }

  onMount(async () => {
    await loadEvents();
    await loadTodos();
  });
</script>

<div class="cal">
  <div class="header">
    <button on:click={prevMonth}>‹</button>
    <span>{year}년 {MONTH_NAMES[month]}</span>
    <button on:click={nextMonth}>›</button>
  </div>

  <div class="grid">
    {#each ['일','월','화','수','목','금','토'] as d}
      <div class="day-label">{d}</div>
    {/each}
    {#each { length: firstDay } as _}
      <div></div>
    {/each}
    {#each days as day}
      {@const isToday = new Date().getDate() === day && new Date().getMonth() === month && new Date().getFullYear() === year}
      {@const evList = eventsOnDay(day)}
      {@const tdList = todosOnDay(day)}
      <div class="day" class:today={isToday}>
        <span>{day}</span>
        <div class="dots">
          {#each evList as _}<span class="dot ev"></span>{/each}
          {#each tdList as _}<span class="dot td"></span>{/each}
        </div>
      </div>
    {/each}
  </div>

  <div class="event-list">
    {#each $events.slice(0, 10) as ev}
      <div class="event-item">
        <span class="ev-dot-inline">●</span>
        <span class="ev-label">{ev.start_at.slice(5, 10)} {ev.title}</span>
        <button on:click={() => deleteEvent(ev.id)}>×</button>
      </div>
    {/each}
  </div>

  {#if showForm}
    <form on:submit|preventDefault={submit} class="form">
      <input bind:value={newTitle} placeholder="일정 제목" autofocus />
      <input bind:value={newStartAt} type="datetime-local" />
      <div class="actions">
        <button type="submit">추가</button>
        <button type="button" on:click={() => (showForm = false)}>취소</button>
      </div>
    </form>
  {:else}
    <button class="add-btn" on:click={() => (showForm = true)}>+ 일정 추가</button>
  {/if}
</div>

<style>
  .cal {
    padding: 12px;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 8px;
    overflow: hidden;
  }

  .header { display: flex; justify-content: space-between; align-items: center; font-weight: bold; flex-shrink: 0; }
  .header button { background: none; border: none; cursor: pointer; font-size: 18px; }

  .grid { display: grid; grid-template-columns: repeat(7, 1fr); gap: 2px; flex-shrink: 0; }
  .day-label { text-align: center; font-size: 10px; color: #aaa; padding: 2px; }
  .day { min-height: 28px; font-size: 11px; padding: 2px; border-radius: 4px; }
  .day.today { background: #e8f4ff; font-weight: bold; }
  .dots { display: flex; flex-wrap: wrap; gap: 1px; margin-top: 1px; }
  .dot { width: 5px; height: 5px; border-radius: 50%; display: inline-block; }
  .dot.ev { background: #4a9eff; }
  .dot.td { background: #ff7043; }

  .event-list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }
  .event-item { display: flex; align-items: center; gap: 4px; font-size: 12px; padding: 4px 0; border-bottom: 1px solid #f0f0f0; }
  .ev-dot-inline { color: #4a9eff; font-size: 8px; flex-shrink: 0; }
  .ev-label { flex: 1; }
  .event-item button { background: none; border: none; cursor: pointer; color: #ccc; flex-shrink: 0; }

  .add-btn {
    width: 100%; padding: 8px;
    background: none; border: 1px dashed #ddd;
    border-radius: 6px; cursor: pointer; color: #888;
    flex-shrink: 0;
  }
  .add-btn:hover { border-color: #4a9eff; color: #4a9eff; }

  .form { display: flex; flex-direction: column; gap: 6px; flex-shrink: 0; }
  .form input { padding: 6px; border: 1px solid #ddd; border-radius: 4px; font-size: 13px; }
  .actions { display: flex; gap: 6px; }
  .actions button { flex: 1; padding: 6px; border: none; border-radius: 4px; cursor: pointer; }
  .actions button[type="submit"] { background: #4a9eff; color: white; }
</style>

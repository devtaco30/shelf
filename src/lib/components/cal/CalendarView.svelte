<script lang="ts">
  import { onMount } from 'svelte';
  import { events, loadEvents, createEvent, deleteEvent } from '$lib/stores/events';

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

  function eventsOnDay(day: number): typeof $events {
    const prefix = `${year}-${String(month + 1).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
    return $events.filter(e => e.start_at.startsWith(prefix));
  }

  function prevMonth() { currentDate = new Date(year, month - 1, 1); }
  function nextMonth() { currentDate = new Date(year, month + 1, 1); }

  async function submit() {
    if (!newTitle.trim() || !newStartAt) return;
    await createEvent(newTitle.trim(), newStartAt, null, 'none');
    newTitle = ''; newStartAt = ''; showForm = false;
  }

  onMount(loadEvents);
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
      <div class="day" class:today={new Date().getDate() === day && new Date().getMonth() === month && new Date().getFullYear() === year}>
        <span>{day}</span>
        {#each eventsOnDay(day) as ev}
          <div class="event-dot" title={ev.title}>·</div>
        {/each}
      </div>
    {/each}
  </div>

  <div class="event-list">
    {#each $events.slice(0, 10) as ev}
      <div class="event-item">
        <span>{ev.start_at.slice(5, 10)} {ev.title}</span>
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
  .cal { padding: 12px; height: 100%; display: flex; flex-direction: column; gap: 8px; overflow-y: auto; }
  .header { display: flex; justify-content: space-between; align-items: center; font-weight: bold; }
  .header button { background: none; border: none; cursor: pointer; font-size: 18px; }
  .grid { display: grid; grid-template-columns: repeat(7, 1fr); gap: 2px; }
  .day-label { text-align: center; font-size: 10px; color: #aaa; padding: 2px; }
  .day { min-height: 28px; font-size: 11px; padding: 2px; border-radius: 4px; }
  .day.today { background: #e8f4ff; font-weight: bold; }
  .event-dot { font-size: 16px; color: #4a9eff; line-height: 1; }
  .event-list { flex: 1; overflow-y: auto; }
  .event-item { display: flex; justify-content: space-between; font-size: 12px; padding: 4px 0; border-bottom: 1px solid #f0f0f0; }
  .event-item button { background: none; border: none; cursor: pointer; color: #ccc; }
  .add-btn { width: 100%; padding: 8px; background: none; border: 1px dashed #ddd; border-radius: 6px; cursor: pointer; color: #888; }
  .form { display: flex; flex-direction: column; gap: 6px; }
  .form input { padding: 6px; border: 1px solid #ddd; border-radius: 4px; font-size: 13px; }
  .actions { display: flex; gap: 6px; }
  .actions button { flex: 1; padding: 6px; border: none; border-radius: 4px; cursor: pointer; }
  .actions button[type="submit"] { background: #4a9eff; color: white; }
</style>

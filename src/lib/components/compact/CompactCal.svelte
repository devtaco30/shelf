<script lang="ts">
  import { onMount } from 'svelte';
  import { events, loadEvents } from '$lib/stores/events';
  import { projects, loadProjects } from '$lib/stores/projects';

  let currentDate = new Date();
  $: year  = currentDate.getFullYear();
  $: month = currentDate.getMonth();
  $: firstDay    = new Date(year, month, 1).getDay();
  $: daysInMonth = new Date(year, month + 1, 0).getDate();
  $: days = Array.from({ length: daysInMonth }, (_, i) => i + 1);

  const MONTH_NAMES = ['1월','2월','3월','4월','5월','6월','7월','8월','9월','10월','11월','12월'];
  const todayPrefix = new Date().toISOString().slice(0, 10);

  function dateStr(day: number): string {
    return `${year}-${String(month + 1).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
  }

  function barsForDay(day: number): { color: string }[] {
    const d = dateStr(day);
    return $projects
      .filter(p => p.start_date && p.end_date && p.start_date <= d && d <= p.end_date)
      .map(p => ({ color: p.color }))
      .slice(0, 3);
  }

  $: todayEvents = $events
    .filter(e => e.start_at.startsWith(todayPrefix))
    .sort((a, b) => a.start_at.localeCompare(b.start_at));

  function formatTime(startAt: string): string {
    const t = startAt.slice(11, 16);
    return t || '';
  }

  function prevMonth(): void { currentDate = new Date(year, month - 1, 1); }
  function nextMonth(): void { currentDate = new Date(year, month + 1, 1); }

  onMount(async () => {
    await Promise.all([loadEvents(), loadProjects()]);
  });
</script>

<div class="compact-cal">
  <!-- Calendar -->
  <div class="cal-header">
    <button class="nav" on:click={prevMonth}>‹</button>
    <span class="month-label">{year}년 {MONTH_NAMES[month]}</span>
    <button class="nav" on:click={nextMonth}>›</button>
  </div>

  <div class="grid">
    {#each ['일','월','화','수','목','금','토'] as d}
      <div class="day-label">{d}</div>
    {/each}
    {#each { length: firstDay } as _}
      <div></div>
    {/each}
    {#each days as day}
      {@const prefix = dateStr(day)}
      {@const isToday = prefix === todayPrefix}
      {@const bars = barsForDay(day)}
      <div class="day" class:today={isToday}>
        <span class="day-num">{day}</span>
        <div class="bars">
          {#each bars as bar}
            <div class="bar" style="background:{bar.color}"></div>
          {/each}
        </div>
      </div>
    {/each}
  </div>

  <!-- Today schedule -->
  <div class="schedule-section">
    <p class="section-title">오늘 일정</p>
    {#each todayEvents as ev}
      <div class="sched-item">
        <span class="sched-dot"></span>
        <span class="sched-time">{formatTime(ev.start_at)}</span>
        <span class="sched-title">{ev.title}</span>
      </div>
    {:else}
      <p class="empty">오늘 일정이 없어요</p>
    {/each}
  </div>
</div>

<style>
  .compact-cal { display: flex; flex-direction: column; height: 100%; }

  .cal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 8px 12px; flex-shrink: 0;
  }
  .month-label { font-size: 12px; font-weight: 600; }
  .nav { background: none; border: none; cursor: pointer; font-size: 16px; color: #888; padding: 0 4px; }
  .nav:hover { color: #111; }

  .grid {
    display: grid; grid-template-columns: repeat(7, 1fr);
    gap: 1px; padding: 0 8px; flex-shrink: 0;
  }
  .day-label { text-align: center; font-size: 9px; color: #aaa; padding: 2px 0; }

  .day {
    display: flex; flex-direction: column; align-items: center;
    padding: 2px 1px; border-radius: 4px; min-height: 26px;
  }
  .day.today { background: rgba(170, 237, 58, 0.15); }
  .day.today .day-num { font-weight: 700; }

  .day-num { font-size: 10px; line-height: 1.2; }

  .bars { display: flex; flex-direction: column; gap: 1px; width: 100%; margin-top: 1px; }
  .bar { height: 2px; border-radius: 1px; width: 100%; }

  .schedule-section {
    padding: 10px 12px; border-top: 0.5px solid #F0F0F0;
    flex: 1; overflow-y: auto;
  }
  .section-title { font-size: 10px; font-weight: 600; color: #888; margin: 0 0 6px; }

  .sched-item {
    display: flex; align-items: center; gap: 6px;
    padding: 4px 0; border-bottom: 0.5px solid #F5F5F5;
  }
  .sched-item:last-child { border: none; }
  .sched-dot { width: 6px; height: 6px; border-radius: 50%; background: #AAED3A; flex-shrink: 0; }
  .sched-time { font-size: 10px; color: #888; width: 32px; flex-shrink: 0; }
  .sched-title { font-size: 11px; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .empty { font-size: 11px; color: #ccc; text-align: center; padding: 12px 0; margin: 0; }
</style>

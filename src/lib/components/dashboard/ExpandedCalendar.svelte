<script lang="ts">
  import { events, createEvent } from '$lib/stores/events';
  import { projects } from '$lib/stores/projects';

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

  function barsForDay(day: number): { color: string; name: string }[] {
    const d = dateStr(day);
    return $projects
      .filter(p => p.start_date && p.end_date && p.start_date <= d && d <= p.end_date)
      .map(p => ({ color: p.color, name: p.name }))
      .slice(0, 2);
  }

  $: todayEvents = $events
    .filter(e => e.start_at.startsWith(todayPrefix))
    .sort((a, b) => a.start_at.localeCompare(b.start_at));

  function getProjectColor(category: string): string {
    const match = $projects.find(p => p.name === category || p.category === category);
    return match ? match.color : '#AAED3A';
  }

  function formatTime(startAt: string): string {
    return startAt.slice(11, 16) || '';
  }

  let newEventTitle = '';
  let newEventDate  = todayPrefix;

  async function handleAddEvent(): Promise<void> {
    if (!newEventTitle.trim() || !newEventDate) return;
    await createEvent(newEventTitle.trim(), `${newEventDate} 09:00:00`, null, 'none');
    newEventTitle = '';
  }

  function prevMonth(): void { currentDate = new Date(year, month - 1, 1); }
  function nextMonth(): void { currentDate = new Date(year, month + 1, 1); }
</script>

<div class="expanded-cal">
  <!-- Left: Calendar + chips + add -->
  <div class="cal-panel">
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

    <!-- Project chips -->
    <div class="proj-chips">
      {#each $projects as proj (proj.id)}
        <span class="chip" style="border-color:{proj.color};color:{proj.color}">
          {proj.name}
        </span>
      {/each}
    </div>

    <!-- Add event -->
    <div class="add-row">
      <input type="date" class="date-input" bind:value={newEventDate} />
      <input
        class="title-input"
        bind:value={newEventTitle}
        placeholder="일정 추가..."
        on:keydown={(e) => e.key === 'Enter' && handleAddEvent()}
      />
      <button class="btn-add" on:click={handleAddEvent}>+</button>
    </div>
  </div>

  <!-- Right: Today schedule -->
  <aside class="schedule-panel">
    <p class="panel-title">오늘 일정</p>
    {#each todayEvents as ev (ev.id)}
      {@const color = getProjectColor(ev.category)}
      <div class="sched-item">
        <div class="sched-bar" style="background:{color}"></div>
        <div class="sched-content">
          <span class="sched-time">{formatTime(ev.start_at)}</span>
          <span class="sched-title">{ev.title}</span>
        </div>
      </div>
    {:else}
      <p class="empty">오늘 일정이 없어요</p>
    {/each}
  </aside>
</div>

<style>
  .expanded-cal {
    display: flex; flex: 1; overflow: hidden;
  }

  /* Left calendar panel */
  .cal-panel {
    flex: 1; display: flex; flex-direction: column;
    padding: 12px 14px; border-right: 0.5px solid #F0F0F0; overflow: hidden;
  }

  .cal-header {
    display: flex; align-items: center; justify-content: space-between;
    margin-bottom: 8px; flex-shrink: 0;
  }
  .month-label { font-size: 13px; font-weight: 600; }
  .nav { background: none; border: none; cursor: pointer; font-size: 18px; color: #888; }
  .nav:hover { color: #111; }

  .grid {
    display: grid; grid-template-columns: repeat(7, 1fr);
    gap: 2px; flex-shrink: 0;
  }
  .day-label { text-align: center; font-size: 10px; color: #aaa; padding: 2px; }

  .day {
    display: flex; flex-direction: column; align-items: center;
    padding: 3px 1px; border-radius: 4px; min-height: 32px;
  }
  .day.today { background: rgba(170, 237, 58, 0.15); }
  .day.today .day-num { font-weight: 700; }
  .day-num { font-size: 11px; line-height: 1.3; }

  .bars { display: flex; flex-direction: column; gap: 1px; width: 90%; margin-top: 2px; }
  .bar  { height: 2px; border-radius: 1px; width: 100%; }

  .proj-chips {
    display: flex; flex-wrap: wrap; gap: 5px;
    margin: 10px 0; flex-shrink: 0;
  }
  .chip {
    padding: 3px 8px; border-radius: 20px; font-size: 10px;
    border: 1px solid; background: transparent; font-weight: 500;
  }

  .add-row {
    display: flex; gap: 6px; margin-top: auto; flex-shrink: 0;
    padding-top: 10px; border-top: 0.5px solid #F0F0F0;
  }
  .date-input {
    border: 0.5px solid #eee; border-radius: 6px;
    padding: 5px 6px; font-size: 11px; background: #F7F7F7;
    outline: none; width: 110px;
  }
  .title-input {
    flex: 1; border: 0.5px solid #eee; border-radius: 6px;
    padding: 5px 8px; font-size: 12px; background: #F7F7F7; outline: none;
  }
  .title-input:focus { border-color: #AAED3A; background: #fff; }
  .btn-add {
    width: 28px; height: 28px; border-radius: 6px;
    background: #AAED3A; border: none; font-size: 18px; line-height: 1;
    cursor: pointer; display: flex; align-items: center; justify-content: center;
    font-weight: 700; color: #111;
  }
  .btn-add:hover { background: #9bde2a; }

  /* Right schedule panel */
  .schedule-panel {
    width: 180px; flex-shrink: 0;
    padding: 12px 12px; overflow-y: auto;
    background: #FAFAFA;
  }
  .panel-title { font-size: 10px; font-weight: 600; color: #aaa; margin: 0 0 10px; }

  .sched-item {
    display: flex; gap: 8px; align-items: flex-start;
    margin-bottom: 10px;
  }
  .sched-bar { width: 3px; border-radius: 2px; min-height: 36px; flex-shrink: 0; }
  .sched-content { display: flex; flex-direction: column; gap: 2px; }
  .sched-time  { font-size: 10px; color: #888; }
  .sched-title { font-size: 12px; font-weight: 500; }

  .empty { font-size: 11px; color: #ccc; text-align: center; margin-top: 20px; }
</style>

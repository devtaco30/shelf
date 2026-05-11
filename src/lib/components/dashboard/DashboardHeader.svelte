<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { todos } from '$lib/stores/todos';
  import { events } from '$lib/stores/events';
  import { projects } from '$lib/stores/projects';
  import { setState } from '$lib/stores/window';

  const today        = new Date();
  const DAY_NAMES    = ['일','월','화','수','목','금','토'];
  const todayPrefix  = today.toISOString().slice(0, 10);
  const dateLabel    = `${today.getFullYear()}.${String(today.getMonth()+1).padStart(2,'0')}.${String(today.getDate()).padStart(2,'0')} ${DAY_NAMES[today.getDay()]}`;

  let profileName = '나';
  let profileBio  = '나의 작업 대시보드';

  $: activeProjectCount = $projects.length;
  $: todayEventCount    = $events.filter(e => e.start_at.startsWith(todayPrefix)).length;
  $: todayDoneCount     = $todos.filter(t => t.done && t.due_date?.startsWith(todayPrefix)).length;

  onMount(async () => {
    const s = await invoke<{ name: string; bio: string }>('get_settings');
    profileName = s.name;
    profileBio  = s.bio;
  });
</script>

<header class="dash-header">
  <div class="profile">
    <div class="avatar">{profileName.charAt(0)}</div>
    <div>
      <div class="profile-name">{profileName} · Shelf</div>
      <div class="profile-bio">{profileBio}</div>
    </div>
  </div>

  <div class="stats">
    <div class="stat"><div class="stat-n">{activeProjectCount}</div><div class="stat-l">진행 프로젝트</div></div>
    <div class="stat"><div class="stat-n">{todayEventCount}</div><div class="stat-l">오늘 일정</div></div>
    <div class="stat"><div class="stat-n">{todayDoneCount}</div><div class="stat-l">완료</div></div>
  </div>

  <span class="date">{dateLabel}</span>

  <div class="actions">
    <button class="btn-compact" on:click={() => setState('panel')}>▼ 컴팩트</button>
    <button class="btn-close"   on:click={() => setState('pill')}>×</button>
  </div>
</header>

<style>
  .dash-header {
    display: flex; align-items: center; gap: 10px;
    padding: 12px 16px; border-bottom: 0.5px solid #F0F0F0; flex-shrink: 0;
  }
  .profile { display: flex; align-items: center; gap: 8px; flex: 1; min-width: 0; }
  .avatar {
    width: 32px; height: 32px; border-radius: 50%;
    background: #111; color: #fff;
    display: flex; align-items: center; justify-content: center;
    font-size: 13px; font-weight: 600; flex-shrink: 0;
  }
  .profile-name { font-size: 13px; font-weight: 600; }
  .profile-bio  { font-size: 10px; color: #aaa; }
  .stats { display: flex; gap: 12px; }
  .stat   { text-align: center; }
  .stat-n { font-size: 18px; font-weight: 700; line-height: 1; }
  .stat-l { font-size: 9px; color: #aaa; }
  .date   { font-size: 11px; color: #aaa; white-space: nowrap; }
  .actions { display: flex; gap: 6px; align-items: center; }
  .btn-compact {
    padding: 4px 10px; border-radius: 6px;
    background: #F5F5F5; border: 0.5px solid #eee; font-size: 10px; cursor: pointer;
  }
  .btn-compact:hover { background: #eee; }
  .btn-close { background: none; border: none; cursor: pointer; color: #bbb; font-size: 16px; }
  .btn-close:hover { color: #333; }
</style>

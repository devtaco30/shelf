<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { createTodo } from '$lib/stores/todos';
  import { projects } from '$lib/stores/projects';
  import { PRIORITY_COLORS } from '$lib/stores/todos';

  const dispatch = createEventDispatcher<{ close: void }>();

  const CATEGORIES = ['작업', '클라이언트', '개인', 'work', '사일'];

  let title       = '';
  let category    = '작업';
  let priority    = 0;
  let dueDate     = new Date().toISOString().slice(0, 10);
  let projectId: number | null = null;
  let loading     = false;

  function selectProject(id: number): void {
    projectId = projectId === id ? null : id;
  }

  async function handleSubmit(): Promise<void> {
    if (!title.trim()) return;
    loading = true;
    try {
      await createTodo(title.trim(), '', dueDate || null, 'none', category, priority);
      dispatch('close');
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'Escape') dispatch('close');
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="overlay" on:click={() => dispatch('close')}>
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal" on:click|stopPropagation>
    <h3 class="modal-title">할 일 추가</h3>

    <!-- Title -->
    <input
      class="title-input"
      bind:value={title}
      placeholder="무엇을 해야 하나요?"
      autofocus
      on:keydown={(e) => e.key === 'Enter' && handleSubmit()}
    />

    <!-- Project chips -->
    {#if $projects.length > 0}
      <div class="section-label">프로젝트</div>
      <div class="project-chips">
        {#each $projects as proj (proj.id)}
          <button
            class="proj-chip"
            class:selected={projectId === proj.id}
            style="--color:{proj.color}"
            on:click={() => selectProject(proj.id)}
          >
            {proj.name}
          </button>
        {/each}
      </div>
    {/if}

    <!-- Priority -->
    <div class="section-label">우선순위</div>
    <div class="priority-row">
      {#each [0, 1, 2, 3] as p}
        {@const color = PRIORITY_COLORS[p]}
        {@const labels = ['없음', '낮음', '보통', '높음']}
        <button
          class="prio-btn" class:selected={priority === p}
          on:click={() => (priority = p)}
        >
          <span
            class="prio-dot"
            style={color ? `background:${color}` : 'background:#ddd'}
          ></span>
          <span class="prio-label">{labels[p]}</span>
        </button>
      {/each}
    </div>

    <!-- Category -->
    <div class="section-label">카테고리</div>
    <div class="cat-row">
      {#each CATEGORIES as cat}
        <button
          class="cat-btn" class:selected={category === cat}
          on:click={() => (category = cat)}
        >{cat}</button>
      {/each}
    </div>

    <!-- Due date -->
    <div class="section-label">기한</div>
    <input type="date" class="date-input" bind:value={dueDate} />

    <!-- Actions -->
    <div class="actions">
      <button class="btn-cancel" on:click={() => dispatch('close')}>취소</button>
      <button class="btn-submit" on:click={handleSubmit} disabled={loading || !title.trim()}>
        {loading ? '추가 중...' : '추가'}
      </button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.4);
    display: flex; align-items: center; justify-content: center;
    z-index: 200;
  }

  .modal {
    background: #fff; border-radius: 14px;
    padding: 20px; width: 300px;
    box-shadow: 0 16px 48px rgba(0,0,0,0.15);
    display: flex; flex-direction: column; gap: 10px;
  }

  .modal-title { margin: 0; font-size: 15px; font-weight: 700; }

  .title-input {
    width: 100%; box-sizing: border-box;
    border: 1px solid #eee; border-radius: 8px;
    padding: 10px 12px; font-size: 14px; outline: none;
  }
  .title-input:focus { border-color: #AAED3A; }

  .section-label { font-size: 10px; font-weight: 600; color: #aaa; margin-bottom: -4px; }

  .project-chips { display: flex; flex-wrap: wrap; gap: 6px; }
  .proj-chip {
    padding: 4px 10px; border-radius: 20px;
    border: 1.5px solid var(--color); color: var(--color);
    background: transparent; cursor: pointer; font-size: 11px; font-weight: 500;
    transition: background 0.1s;
  }
  .proj-chip.selected { background: var(--color); color: #fff; }

  .priority-row { display: flex; gap: 8px; }
  .prio-btn {
    display: flex; flex-direction: column; align-items: center; gap: 4px;
    padding: 6px 10px; border-radius: 8px;
    border: 1.5px solid #eee; background: transparent; cursor: pointer;
    transition: border-color 0.1s;
  }
  .prio-btn.selected { border-color: #111; }
  .prio-dot { width: 10px; height: 10px; border-radius: 50%; }
  .prio-label { font-size: 9px; color: #888; }

  .cat-row { display: flex; flex-wrap: wrap; gap: 5px; }
  .cat-btn {
    padding: 4px 9px; border-radius: 6px;
    border: 1px solid #eee; background: transparent; cursor: pointer;
    font-size: 11px; color: #555;
    transition: border-color 0.1s, background 0.1s;
  }
  .cat-btn.selected { border-color: #111; background: #111; color: #fff; }

  .date-input {
    border: 1px solid #eee; border-radius: 8px;
    padding: 7px 10px; font-size: 12px; outline: none; width: 100%;
    box-sizing: border-box;
  }

  .actions { display: flex; gap: 8px; margin-top: 4px; }
  .btn-cancel {
    flex: 1; padding: 9px; border-radius: 8px;
    border: 1px solid #eee; background: #F5F5F5; cursor: pointer;
    font-size: 13px;
  }
  .btn-submit {
    flex: 1; padding: 9px; border-radius: 8px;
    border: none; background: #111; color: #fff; cursor: pointer;
    font-size: 13px; font-weight: 600;
  }
  .btn-submit:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-cancel:hover { background: #eee; }
  .btn-submit:not(:disabled):hover { background: #333; }
</style>

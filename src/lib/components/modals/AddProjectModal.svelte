<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { createProject, PROJECT_COLORS } from '$lib/stores/projects';

  const dispatch = createEventDispatcher<{ close: void }>();

  let name      = '';
  let color     = PROJECT_COLORS[0];
  let category  = '개인';
  let startDate = '';
  let endDate   = '';
  let loading   = false;

  async function handleSubmit(): Promise<void> {
    if (!name.trim()) return;
    loading = true;
    try {
      await createProject(name.trim(), color, category, startDate || null, endDate || null);
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
    <h3 class="modal-title">새 프로젝트</h3>

    <!-- Name -->
    <input
      class="name-input"
      bind:value={name}
      placeholder="프로젝트 이름"
      autofocus
      on:keydown={(e) => e.key === 'Enter' && handleSubmit()}
    />

    <!-- Color swatches -->
    <div class="section-label">색상</div>
    <div class="color-grid">
      {#each PROJECT_COLORS as c}
        <button
          class="swatch" class:selected={color === c}
          style="background:{c}"
          on:click={() => (color = c)}
          aria-label={c}
        ></button>
      {/each}
    </div>

    <!-- Category toggle -->
    <div class="section-label">유형</div>
    <div class="toggle-row">
      <button
        class="toggle-btn" class:active={category === '개인'}
        on:click={() => (category = '개인')}
      >개인</button>
      <button
        class="toggle-btn" class:active={category === '회사'}
        on:click={() => (category = '회사')}
      >회사</button>
    </div>

    <!-- Date range -->
    <div class="section-label">기간</div>
    <div class="date-row">
      <input type="date" class="date-input" bind:value={startDate} />
      <span class="date-sep">~</span>
      <input type="date" class="date-input" bind:value={endDate} />
    </div>

    <!-- Actions -->
    <div class="actions">
      <button class="btn-cancel" on:click={() => dispatch('close')}>취소</button>
      <button class="btn-submit" on:click={handleSubmit} disabled={loading || !name.trim()}>
        {loading ? '추가 중...' : '만들기'}
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
    padding: 20px; width: 280px;
    box-shadow: 0 16px 48px rgba(0,0,0,0.15);
    display: flex; flex-direction: column; gap: 10px;
  }

  .modal-title { margin: 0; font-size: 15px; font-weight: 700; }

  .name-input {
    width: 100%; box-sizing: border-box;
    border: 1px solid #eee; border-radius: 8px;
    padding: 10px 12px; font-size: 14px; outline: none;
  }
  .name-input:focus { border-color: #AAED3A; }

  .section-label { font-size: 10px; font-weight: 600; color: #aaa; margin-bottom: -4px; }

  .color-grid { display: flex; flex-wrap: wrap; gap: 8px; }
  .swatch {
    width: 26px; height: 26px; border-radius: 50%;
    border: 2px solid transparent; cursor: pointer; padding: 0;
    transition: transform 0.1s, border-color 0.1s;
  }
  .swatch:hover    { transform: scale(1.1); }
  .swatch.selected { border-color: #111; transform: scale(1.15); }

  .toggle-row { display: flex; border: 1px solid #eee; border-radius: 8px; overflow: hidden; }
  .toggle-btn {
    flex: 1; padding: 8px; border: none; background: transparent;
    cursor: pointer; font-size: 13px; transition: background 0.1s;
  }
  .toggle-btn.active { background: #111; color: #fff; font-weight: 600; }
  .toggle-btn:not(.active):hover { background: #F5F5F5; }

  .date-row { display: flex; align-items: center; gap: 6px; }
  .date-input {
    flex: 1; border: 1px solid #eee; border-radius: 8px;
    padding: 7px 8px; font-size: 11px; outline: none; box-sizing: border-box;
  }
  .date-sep { color: #aaa; font-size: 12px; flex-shrink: 0; }

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

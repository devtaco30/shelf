<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { windowState } from '$lib/stores/window';
  import { toast, hideToast } from '$lib/stores/toast';
  import { showAddTodoModal, editingTodo } from '$lib/stores/todos';
  import Pill from '$lib/components/Pill.svelte';
  import AddTodoModal from '$lib/components/modals/AddTodoModal.svelte';
  import CompactPanel from '$lib/components/CompactPanel.svelte';
  import ExpandedDashboard from '$lib/components/dashboard/ExpandedDashboard.svelte';

  let { children } = $props();

  let vaultLockTimer: ReturnType<typeof setTimeout>;

  function resetVaultTimer(): void {
    clearTimeout(vaultLockTimer);
    vaultLockTimer = setTimeout(async () => {
      const { lockVault } = await import('$lib/stores/vault');
      await lockVault();
    }, 5 * 60 * 1000);
  }

  async function startDrag(e: MouseEvent): Promise<void> {
    if (e.button !== 0) return;
    await getCurrentWindow().startDragging();
  }

  onMount(async () => {
    window.addEventListener('mousemove', resetVaultTimer);
    window.addEventListener('keydown', resetVaultTimer);
    resetVaultTimer();
  });
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="app" onmousedown={startDrag}>
  <Pill />

  {#if $windowState === 'panel'}
    <div class="panel-wrapper">
      <CompactPanel />
    </div>
  {:else if $windowState === 'expanded'}
    <div class="panel-wrapper">
      <ExpandedDashboard />
    </div>
  {/if}

  {#if $showAddTodoModal}
    <AddTodoModal on:close={() => showAddTodoModal.set(false)} />
  {/if}
  {#if $editingTodo}
    <AddTodoModal todo={$editingTodo} on:close={() => editingTodo.set(null)} />
  {/if}

  <!-- 삭제 토스트 (최상위 레이어) -->
  {#if $toast}
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      onmousedown={(e) => e.stopPropagation()}
      style="position:absolute;bottom:20px;left:50%;transform:translateX(-50%);background:#1E1E2E;color:#fff;padding:9px 16px;border-radius:10px;font-size:12px;display:flex;align-items:center;gap:10px;white-space:nowrap;z-index:200;"
    >
      <span>{$toast.msg}</span>
      {#if $toast.onUndo}
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <div
          onclick={() => { $toast?.onUndo?.(); hideToast(); }}
          style="background:#AAED3A;color:#111;border-radius:6px;padding:3px 9px;font-size:11px;font-weight:700;cursor:pointer;"
        >되돌리기</div>
      {/if}
    </div>
  {/if}
</div>

<style>
  :global(html), :global(body) {
    margin: 0;
    overflow: hidden;
    font-family: -apple-system, 'Apple SD Gothic Neo', sans-serif;
    background: transparent;
  }

  .app {
    position: relative;
    width: 100%;
    height: 100vh;
    background: transparent;
    overflow: hidden;
  }

  .panel-wrapper {
    position: absolute;
    left: 62px;
    top: 50%;
    transform: translateY(-50%);
  }

</style>

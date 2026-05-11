<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher<{ close: void }>();
</script>

<div class="overlay" on:click={() => dispatch('close')} role="dialog" aria-modal="true">
  <div class="modal" on:click|stopPropagation role="presentation">
    <p style="padding:20px;font-size:13px;color:#aaa">프로젝트 추가 (구현 예정)</p>
    <button on:click={() => dispatch('close')}>닫기</button>
  </div>
</div>

<style>
  .overlay { position:fixed;inset:0;background:rgba(0,0,0,0.3);display:flex;align-items:center;justify-content:center;z-index:100; }
  .modal { background:#fff;border-radius:12px;padding:16px;min-width:280px; }
  button { padding:6px 14px;border:none;border-radius:6px;background:#111;color:#fff;cursor:pointer;width:100%;margin-top:8px; }
</style>

<script lang="ts">
  import { vaultInitialized, setupVault, unlockVault } from '$lib/stores/vault';

  let password = '';
  let confirm = '';
  let error = '';
  let loading = false;

  async function handleSubmit() {
    error = '';
    loading = true;
    try {
      if (!$vaultInitialized) {
        if (password !== confirm) { error = '비밀번호가 일치하지 않습니다'; return; }
        if (password.length < 8) { error = '비밀번호는 8자 이상이어야 합니다'; return; }
        await setupVault(password);
      } else {
        await unlockVault(password);
      }
    } catch (e) {
      error = '비밀번호가 올바르지 않습니다';
    } finally {
      loading = false;
      password = '';
      confirm = '';
    }
  }
</script>

<div class="lock-screen">
  <div class="icon">🔒</div>
  <h2>{$vaultInitialized ? 'Vault 잠금 해제' : 'Vault 설정'}</h2>

  <form on:submit|preventDefault={handleSubmit}>
    <input
      type="password"
      bind:value={password}
      placeholder="비밀번호"
      autofocus
    />
    {#if !$vaultInitialized}
      <input
        type="password"
        bind:value={confirm}
        placeholder="비밀번호 확인"
      />
    {/if}
    {#if error}
      <p class="error">{error}</p>
    {/if}
    <button type="submit" disabled={loading}>
      {loading ? '처리 중...' : $vaultInitialized ? '잠금 해제' : '설정 완료'}
    </button>
  </form>
</div>

<style>
  .lock-screen { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; gap: 12px; padding: 24px; }
  .icon { font-size: 48px; }
  h2 { margin: 0; font-size: 16px; color: #333; }
  form { display: flex; flex-direction: column; gap: 8px; width: 100%; }
  input { padding: 10px; border: 1px solid #ddd; border-radius: 8px; font-size: 14px; text-align: center; }
  button { padding: 10px; background: #4a9eff; color: white; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; }
  button:disabled { opacity: 0.6; }
  .error { color: #f55; font-size: 12px; text-align: center; margin: 0; }
</style>

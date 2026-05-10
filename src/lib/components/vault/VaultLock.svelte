<script lang="ts">
  import { vaultInitialized, setupVault, unlockVault } from '$lib/stores/vault';

  let password = '';
  let confirm = '';
  let error = '';
  let loading = false;

  async function handleTouchId() {
    error = '';
    loading = true;
    try {
      await unlockVault(); // 비밀번호 없이 → Touch ID 시도
    } catch (e) {
      error = 'Touch ID 인증에 실패했습니다. 비밀번호를 입력해 주세요.';
    } finally {
      loading = false;
    }
  }

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
  <h2>{$vaultInitialized ? 'Vault 잠금 해제' : 'Vault 초기 설정'}</h2>
  {#if !$vaultInitialized}
    <p class="hint">처음 한 번만 비밀번호를 설정합니다.<br>이후에는 Touch ID로 잠금 해제할 수 있습니다.</p>
  {/if}

  {#if $vaultInitialized}
    <button class="touchid-btn" on:click={handleTouchId} disabled={loading}>
      {loading ? '인증 중...' : '👆 Touch ID로 잠금 해제'}
    </button>
    <div class="divider"><span>또는</span></div>
  {/if}

  <form on:submit|preventDefault={handleSubmit}>
    <input
      type="password"
      bind:value={password}
      placeholder="비밀번호"
      autofocus={!$vaultInitialized}
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
      {loading ? '처리 중...' : $vaultInitialized ? '비밀번호로 잠금 해제' : '설정 완료'}
    </button>
  </form>
</div>

<style>
  .lock-screen { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; gap: 10px; padding: 24px; }
  .icon { font-size: 40px; }
  h2 { margin: 0; font-size: 15px; color: #333; }
  form { display: flex; flex-direction: column; gap: 8px; width: 100%; }
  input { padding: 10px; border: 1px solid #ddd; border-radius: 8px; font-size: 14px; text-align: center; }
  button { padding: 10px; background: #4a9eff; color: white; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; }
  button:disabled { opacity: 0.6; }
  .touchid-btn { width: 100%; background: #1e1e2e; font-size: 14px; }
  .touchid-btn:hover:not(:disabled) { background: #2e2e4e; }
  .divider { display: flex; align-items: center; gap: 8px; width: 100%; color: #aaa; font-size: 12px; }
  .divider::before, .divider::after { content: ''; flex: 1; height: 1px; background: #eee; }
  .hint { font-size: 11px; color: #888; text-align: center; margin: 0; line-height: 1.5; }
  .error { color: #f55; font-size: 12px; text-align: center; margin: 0; }
</style>

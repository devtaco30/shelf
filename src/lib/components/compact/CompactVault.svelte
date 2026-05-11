<script lang="ts">
  import { onMount } from 'svelte';
  import { vaultUnlocked, checkVaultState } from '$lib/stores/vault';
  import VaultLock from '$lib/components/vault/VaultLock.svelte';
  import VaultList from '$lib/components/vault/VaultList.svelte';

  onMount(checkVaultState);
</script>

<div class="compact-vault">
  {#if $vaultUnlocked}
    <VaultList />
  {:else}
    <VaultLock />
  {/if}
</div>

<style>
  .compact-vault { height: 100%; overflow-y: auto; }
</style>

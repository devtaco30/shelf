<script lang="ts">
  import { vaultUnlocked, vaultInitialized, checkVaultState } from '$lib/stores/vault';
  import VaultLock from '$lib/components/vault/VaultLock.svelte';
  import VaultList from '$lib/components/vault/VaultList.svelte';
  import { onMount } from 'svelte';

  onMount(checkVaultState);
</script>

<div class="expanded-vault">
  {#if $vaultUnlocked}
    <VaultList />
  {:else}
    <VaultLock />
  {/if}
</div>

<style>
  .expanded-vault { flex: 1; overflow-y: auto; }
</style>

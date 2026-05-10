<script lang="ts">
  import { onMount } from 'svelte';
  import { vaultUnlocked, vaultInitialized, checkVaultState } from '$lib/stores/vault';
  import VaultLock from '$lib/components/vault/VaultLock.svelte';
  import VaultList from '$lib/components/vault/VaultList.svelte';

  onMount(checkVaultState);
</script>

{#if $vaultUnlocked}
  <VaultList />
{:else}
  <VaultLock />
{/if}

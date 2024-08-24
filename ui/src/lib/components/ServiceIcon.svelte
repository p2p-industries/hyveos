<script lang="ts">
  import Nginx from '$lib/components/icons/Nginx.svelte';
  import Redis from '$lib/components/icons/Redis.svelte';
  import PostgreSql from '$lib/components/icons/PostgreSQL.svelte';
  import Apache from '$lib/components/icons/Apache.svelte';
  export let name: string;
</script>

{#if name.startsWith('nginx')}
  <Nginx />
{:else if name.startsWith('redis')}
  <Redis />
{:else if name.startsWith('postgres')}
  <PostgreSql />
{:else if name.startsWith('apache') || name.startsWith('httpd')}
  <Apache />
{/if}

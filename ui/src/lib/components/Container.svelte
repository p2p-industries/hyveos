<script lang="ts">
  import ServiceIcon from './ServiceIcon.svelte';

  interface Props {
    name: string;
    status: 'running' | 'stopped';
  }

  let { name, status }: Props = $props();

  let isRunning = $derived(status === 'running');
</script>

<div class="card bg-base-100 w-fit shadow-lg hover:shadow-xl">
  <figure>
    <div class="w-20 h-20">
      <ServiceIcon {name} />
    </div>
  </figure>

  <div class="card-body">
    <h2 class="card-title">{name}</h2>
    <div class="font-semibold" class:text-green-500={isRunning} class:text-red-500={!isRunning}>
      {status}
    </div>
  </div>
</div>

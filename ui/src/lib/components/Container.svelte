<script lang="ts">
  import ServiceIcon from './ServiceIcon.svelte';

  type Status = 'running' | 'stopped';
  export let name: string;
  export let status: Status;
</script>

<div class="card bg-base-100 w-fit shadow-lg hover:shadow-xl">
  <figure>
    <div class="w-20 h-20">
      <ServiceIcon {name} />
    </div>
  </figure>

  <div class="card-body">
    <h2 class="card-title">{name}</h2>
    <div class={`font-semibold ${status === 'running' ? 'text-green-500' : 'text-red-500'}`}>
      {status}
    </div>
  </div>
</div>

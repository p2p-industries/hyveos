<script lang="ts">
  import { DeviceStatusTag, type DeviceType } from '$lib/types';
  import DeviceIcon from '$lib/assets/device-icon.png';
  import Container from './Container.svelte';

  interface Props {
    device: DeviceType;
  }

  let { device }: Props = $props();
</script>

<details class="border rounded-lg p-4 m-2 cursor-pointer hover:shadow-lg hover:border-blue-500">
  <summary class="flex items-center justify-between">
    <div class="flex items-center">
      <img src={DeviceIcon} alt="Device pictogram" class="w-16 h-16 object-cover mr-4" />
      <div class="text-lg font-medium">{device.name}</div>
    </div>
    {#if device.status.tag === DeviceStatusTag.Ok}
      <div class="w-4 h-4 rounded-full bg-success" role="status"></div>
    {:else if device.status.tag === DeviceStatusTag.Error}
      {@const error = device.status.error}
      <div
        class="w-4 h-4 rounded-full bg-error tooltip tooltip-left"
        role="status"
        data-tip={error}
></div>
    {/if}
  </summary>
  <div class="flex justify-center overflow-x-auto space-x-4 py-2">
    {#each device.dockerImages as { name, status }}
      <Container {name} {status} />
    {/each}
  </div>
</details>

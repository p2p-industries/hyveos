<script lang="ts">
  import Device from '$lib/components/Device.svelte';
  import { faStop, faPlay, faChevronDown } from '@fortawesome/free-solid-svg-icons';
  import { FontAwesomeIcon } from '@fortawesome/svelte-fontawesome';
  import type { PageData } from './$types';

  export let data: PageData;

  $: devices = data.devices;

  let allSelected = false;
  let showDeployDialog = false;
  let dockerImageInput = '';

  function toggleAllCheckboxes() {
    allSelected = !allSelected;
    devices.forEach((device) => {
      const checkbox = document.getElementById(device.id) as HTMLInputElement;
      checkbox.checked = allSelected;
    });
  }

  function handleDeploy() {
    console.log(`Deploying ${dockerImageInput} to selected devices`);
    // Implement the deployment logic
  }

  function handleAction(action: string) {
    // Implement the action handlers (e.g., Stop)
    console.log(`${action} action triggered`);
  }
</script>

<div class="p-4 flex flex-col">
  <div class="flex flex-col space-y-2">
    <div class="flex items-center justify-between p-2 border-b-2">
      <div class="flex items-center">
        <input type="checkbox" on:click={toggleAllCheckboxes} />
        <span class="ml-2 font-medium">Devices</span>
      </div>
      <div class="flex items-center space-x-4">
        <button
          class="text-gray-700 hover:text-gray-900 border rounded px-2 py-1"
          on:click={() => handleAction('Stop')}
        >
          <FontAwesomeIcon icon={faStop} />
        </button>
        <div class="relative">
          <div class="flex border rounded px-2 py-1 space-x-3">
            <button class="text-gray-700 hover:text-gray-900" on:click={() => handleDeploy()}>
              <FontAwesomeIcon icon={faPlay} />
            </button>
            <button
              class="text-gray-700 hover:text-gray-900"
              on:click={() => (showDeployDialog = !showDeployDialog)}
            >
              <FontAwesomeIcon icon={faChevronDown} />
            </button>
          </div>
          {#if showDeployDialog}
            <div class="absolute right-0 mt-2 w-64 bg-white border rounded shadow-lg p-4 z-10">
              <label class="block text-gray-700 text-sm font-bold mb-2" for="dockerImage"
                >Docker Image</label
              >
              <input
                class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                id="dockerImage"
                type="text"
                bind:value={dockerImageInput}
                placeholder="nginx:latest"
              />
            </div>
          {/if}
        </div>
      </div>
    </div>

    {#each devices as device}
      <Device {device} />
    {/each}
  </div>
</div>

<style>
  .relative .absolute {
    z-index: 10;
  }
  .border {
    border: 1px solid #ddd;
  }
</style>

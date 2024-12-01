<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  interface Props {
    children?: import('svelte').Snippet;
  }

  let { children }: Props = $props();

  const DASHBOARDS = [
    { name: 'Devices', path: '/devices' },
    { name: 'Data', path: '/data' },
    { name: 'Mesh', path: '/mesh' }
  ];
</script>

<div class="flex flex-row h-screen">
  <nav class="bg-gray-800 text-white h-full">
    <ul>
      {#each DASHBOARDS as { name, path }}
        {@const current = $page.url.pathname.startsWith(path)}
        <li style:--bg-color={current ? 'rgb(71 85 105)' : ''} class="cursor-pointer">
          <a href={path} class="p-4">
            <span class="text-xl">
              {name}
            </span>
          </a>
        </li>
      {/each}
    </ul>
  </nav>
  <main class="grow">
    {@render children?.()}
  </main>
</div>

<style>
  nav {
    width: 200px;
  }

  li {
    background-color: var(--bg-color);
  }

  li:hover {
    --bg-color: rgb(44 53 67);
  }
</style>

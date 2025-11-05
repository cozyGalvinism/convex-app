<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { launchInstance, scan, showInstance } from "../lib/api";
  import type { Instance, ScanResult } from "../lib/api";
  import { watchGamepad } from "$lib/gamepad";
  import GroupStrip from "$lib/GroupStrip.svelte";

  let data = $state({ instances: [], groups: {} } as ScanResult);
  let searchEl: HTMLInputElement;
  let filtered: Instance[] = $state([]);
  let group: string | "All" = $state("All");
  let search = $state("");
  let focus = $state(0);
  let cols = $state(5);
  let lastMethod: "controller" | "keyboard" = $state("controller");

  const recompute = () => {
    let ids = data.instances;
    if (group !== "All") {
      const set = new Set(data.groups[group] || []);
      ids = ids.filter((i) => set.has(i.id));
    }

    if (search.trim()) {
      const q = search.toLowerCase();
      ids = ids.filter(
        (i) =>
          i.name.toLowerCase().includes(q) || i.id.toLowerCase().includes(q),
      );
    }

    filtered = ids;
    focus = Math.min(focus, Math.max(0, filtered.length - 1));
  };

  const move = (dx: number, dy: number) => {
    if (!filtered.length) return;
    const n = filtered.length;
    const row = Math.floor(focus / cols);
    const col = focus % cols;
    const nr = Math.max(0, Math.min(row + dy, Math.floor((n - 1) / cols)));
    const nc = Math.max(0, Math.min(col + dx, cols - 1));
    let ni = nr * cols + nc;
    if (ni >= n) ni = n - 1;
    focus = ni;
    scrollIntoView();
  };

  const actLaunch = () => filtered[focus] && launchInstance(filtered[focus].id);
  const openInPrism = () => filtered[focus] && showInstance(filtered[focus].id);

  const scrollIntoView = () => {
    const el = document.querySelector(`[data-idx="${focus}"]`);
    el?.scrollIntoView({
      block: "nearest",
      inline: "nearest",
      behavior: "smooth",
    });
  };

  const bumpGroup = (dir: number) => {
    const names = ["All", ...Object.keys(data.groups).sort()];
    const idx = names.indexOf(group);
    const ni = (idx + dir + names.length) % names.length;
    group = names[ni];
    recompute();
    focus = 0;
    scrollIntoView();
  };

  const recalcCols = () => {
    const ww = window.innerWidth;
    const target = 360;
    cols = Math.max(1, Math.floor(ww / target));
  };

  const onResize = () => {
    updateBars();
    recalcCols();
    scrollIntoView();
  };

  const focusSearchBar = () => {
    const el = document.getElementById("search");
    el?.focus();
  };

  onMount(() => {
    updateBars();
    const ro = new ResizeObserver(() => updateBars());
    const header = document.getElementById("topbar");
    const footer = document.getElementById("footer");
    if (header) ro.observe(header);
    if (footer) ro.observe(footer);

    recalcCols();
    window.addEventListener("resize", onResize);

    scan().then((result) => {
      data = result;
      recompute();
    });

    const dispose = watchGamepad((e) => {
      lastMethod = "controller";
      if (e === "left") move(-1, 0);
      if (e === "right") move(1, 0);
      if (e === "up") move(0, -1);
      if (e === "down") move(0, 1);
      if (e === "a") actLaunch();
      if (e === "x") openInPrism();
      if (e === "y") focusSearchBar();
      if (e === "lb") bumpGroup(-1);
      if (e === "rb") bumpGroup(1);
    });
    return () => {
      window.removeEventListener("resize", onResize);
      ro.disconnect();
      dispose();
    };
  });

  function updateBars() {
    const header = document.getElementById("topbar");
    const footer = document.getElementById("footer");
    const hh = header?.getBoundingClientRect().height ?? 0;
    const fh = footer?.getBoundingClientRect().height ?? 0;
    document.documentElement.style.setProperty("--header-h", `${hh}px`);
    document.documentElement.style.setProperty("--footer-h", `${fh}px`);
  }

  $effect(() => {
    if (group !== undefined) {
      untrack(() => {
        recompute();
        focus = 0;
      });
    }
  });
</script>

<div
  class="sticky top-0 z-10 backdrop-blur bg-neutral-900/80 border-b border-neutral-800"
  id="topbar"
>
  <div
    class="max-w-7xl mx-auto px-4 py-3 flex gap-3 justify-between items-center"
  >
    <div class="font-semibold text-lg whitespace-nowrap">Convex</div>

    <div class="flex-1 flex justify-center">
      <GroupStrip bind:active={group} groups={data.groups} />
    </div>

    <div class="flex items-center gap-2">
      <input
        id="search"
        bind:this={searchEl}
        class="bg-neutral-800 rounded px-3 py-1 w-64"
        bind:value={search}
        placeholder="Search…"
        oninput={() => {
          recompute();
          focus = 0;
        }}
      />
      <button
        class="bg-neutral-800 hover:bg-neutral-700 rounded px-3 py-1"
        onclick={async () => {
          data = await scan();
          recompute();
        }}>Reload</button
      >
    </div>
  </div>
</div>

<main id="content" class="max-w-7xl mx-auto px-4 py-6">
  {#if filtered.length === 0}
    <div class="text-neutral-400">
      No instances found. Add an instance in Prism Launcher!
    </div>
  {:else}
    <div
      class="grid gap-5"
      style={`grid-template-columns: repeat(${cols}, minmax(0, 1fr));`}
    >
      {#each filtered as inst, i}
        <button
          class="tile group relative aspect-video rounded-2xl overflow-hidden bg-neutral-800 ring-2 transition {i ===
          focus
            ? 'ring-sky-400'
            : 'ring-transparent'}"
          data-idx={i}
          onmouseenter={() => (focus = i)}
          onclick={actLaunch}
        >
          {#if inst.icon_b64}
            <img
              src={inst.icon_b64}
              alt=""
              class="absolute inset-0 w-full h-full object-contain p-6 opacity-90 group-hover:opacity-100"
            />
          {/if}
          <div
            class="absolute inset-x-0 bottom-0 p-3 bg-linear-to-t from-neutral-900/90 via-neutral-900/30 to-transparent"
          >
            <div class="font-semibold truncate">{inst.name}</div>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</main>

<div
  class="fixed bottom-0 inset-x-0 z-20 border-t border-neutral-800 bg-neutral-900/90 backdrop-blur"
  id="footer"
>
  <div
    class="max-w-7xl mx-auto px-4 py-2 flex flex-wrap gap-x-6 gap-y-2 items-center text-sm text-neutral-300"
  >
    {#if lastMethod === "controller"}
      <div class="flex items-center gap-2">
        <span class="btn">A</span><span class="muted">Launch</span>
        <span class="sep">|</span>
        <span class="btn">X</span><span class="muted">Open in Prism</span>
        <span class="sep">|</span>
        <span class="btn">Y</span><span class="muted">Focus Search</span>
        <span class="sep">|</span>
        <span class="kbd">LB</span>/<span class="kbd">RB</span><span
          class="muted">Switch Group</span
        >
      </div>
    {:else}
      <div class="flex items-center gap-2">
        <span class="kbd">Enter</span><span class="muted">Launch</span>
        <span class="sep">|</span>
        <span class="kbd">/</span><span class="muted">Search</span>
        <span class="sep">|</span>
        <span class="kbd">Q</span>/<span class="kbd">E</span><span class="muted"
          >Switch Group</span
        >
      </div>
    {/if}

    <div class="ml-auto text-neutral-500">
      Press <span class="btn steam">Steam</span> to quit
    </div>
  </div>
</div>

<svelte:window
  onkeydown={(e) => {
    lastMethod = "keyboard";

    if (e.key === "ArrowLeft") move(-1, 0);
    if (e.key === "ArrowRight") move(1, 0);
    if (e.key === "ArrowUp") move(0, -1);
    if (e.key === "ArrowDown") move(0, 1);
    if (e.key === "Enter") actLaunch();
    if (e.key === "q") bumpGroup(-1);
    if (e.key === "e") bumpGroup(1);
  }}
/>

<style lang="postcss">
  @reference "tailwindcss";
  button:focus {
    outline: none;
  }

  .kbd {
    @apply inline-flex items-center justify-center rounded border border-neutral-700 bg-neutral-800 px-2 py-0.5 text-neutral-100;
  }
  .btn.steam {
    @apply inline-flex items-center justify-center rounded-full border border-neutral-700 bg-neutral-800 px-4 py-0.5 text-neutral-100;
  }
  .btn {
    @apply inline-flex items-center justify-center px-2 py-0.5 font-semibold border border-neutral-700 bg-neutral-800 rounded-full;
  }
  .muted {
    @apply text-neutral-400;
  }
  .sep {
    @apply text-neutral-600 px-1;
  }
</style>

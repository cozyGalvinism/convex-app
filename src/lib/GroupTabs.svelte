<script lang="ts">
    import { onMount, tick } from "svelte";

    export let groups: Record<string, string[]> = {};
    export let active: string | "All" = "All";

    let scroller: HTMLDivElement;
    let pills: HTMLDivElement[] = [];

    const names = () => ["All", ...Object.keys(groups).sort()];

    // center the active pill in view
    async function centerActive() {
        await tick();
        const idx = names().indexOf(active);
        const el = pills[idx];
        if (!el || !scroller) return;
        const elRect = el.getBoundingClientRect();
        const scRect = scroller.getBoundingClientRect();
        const offset =
            elRect.left + elRect.width / 2 - (scRect.left + scRect.width / 2);
        scroller.scrollLeft += offset;
    }

    // horizontally scroll with mouse wheel/trackpad too
    const wheelHoriz = (e: WheelEvent) => {
        if (!scroller) return;
        scroller.scrollLeft +=
            Math.abs(e.deltaY) > Math.abs(e.deltaX) ? e.deltaY : e.deltaX;
    };

    onMount(centerActive);
    $: active, centerActive(); // re-center when active changes
</script>

<div class="flex items-center gap-2 px-2 py-2">
    <!-- LB hint -->
    <span class="hidden md:inline kbd">LB</span>

    <div
        bind:this={scroller}
        class="relative flex overflow-x-auto no-scrollbar gap-2 px-1"
        on:wheel|passive={wheelHoriz}
        aria-label="Groups position"
    >
        {#each names() as g, i}
            <div
                bind:this={pills[i]}
                class="px-3 py-1.5 rounded-full text-sm whitespace-nowrap select-none
               transition-colors
               {active === g
                    ? 'bg-neutral-200 text-neutral-900 font-semibold'
                    : 'bg-neutral-800/70 text-neutral-300'}"
                style="pointer-events:none"
            >
                {g}
            </div>
        {/each}
    </div>

    <!-- RB hint -->
    <span class="hidden md:inline kbd">RB</span>
</div>

<style>
    /* Optional: hide scrollbars */
    .no-scrollbar::-webkit-scrollbar {
        display: none;
    }
    .no-scrollbar {
        -ms-overflow-style: none;
        scrollbar-width: none;
    }
</style>

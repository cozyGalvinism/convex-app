import { invoke } from "@tauri-apps/api/core";
import { Window } from "@tauri-apps/api/window";

export async function showDeckKeyboardFor(el: HTMLElement) {
    const r = el.getBoundingClientRect();
    const win = Window.getCurrent();
    const winPos = await win.outerPosition();
    const dpr = window.devicePixelRatio || 1;

    const x = Math.round((winPos.x + r.left) * dpr);
    const y = Math.round((winPos.y + r.top) * dpr);
    const w = Math.round(r.width * dpr);
    const h = Math.round(r.height * dpr);

    await invoke("show_steam_deck_keyboard", { x, y, width: w, height: h });
}

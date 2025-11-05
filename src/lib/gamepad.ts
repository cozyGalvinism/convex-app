export type PadEvent = "left" | "right" | "up" | "down" | "a" | "x" | "y" | "lb" | "rb";

export type RepeatOptions = {
    pollHz?: number;           // how often to sample the gamepad (default 120Hz)
    deadzone?: number;         // stick deadzone
    initialDelayMs?: number;   // delay before first repeat
    repeatMsStart?: number;    // starting repeat interval after delay
    repeatMsMin?: number;      // fastest it can go
    accelMsPerSec?: number;    // shave ms per second held
    axisBoostFactor?: number;  // stronger tilt => faster repeats
};

type Dir = "left" | "right" | "up" | "down";
const DIRS: Dir[] = ["left", "right", "up", "down"];

export function watchGamepad(
    on: (e: PadEvent) => void,
    opts: RepeatOptions = {}
) {
    const pollHz = opts.pollHz ?? 120;
    const intervalMs = 1000 / pollHz;
    const dead = opts.deadzone ?? 0.25;
    const initialDelay = opts.initialDelayMs ?? 220;
    const repeatStart = opts.repeatMsStart ?? 90;
    const repeatMin = opts.repeatMsMin ?? 28;
    const accelPerSec = opts.accelMsPerSec ?? 140;
    const axisBoost = opts.axisBoostFactor ?? 0.7;

    // Per-direction hold state
    const hold: Record<Dir, {
        down: boolean;
        heldMs: number;
        nextFireMs: number;
        curRepeatMs: number;
    }> = Object.fromEntries(DIRS.map(d => [d, { down: false, heldMs: 0, nextFireMs: 0, curRepeatMs: repeatStart }])) as any;

    // Previous button states for edge detection
    let prevButtons: boolean[] = [];
    // We also track previous directional "want" to catch edges from axes/hat
    let prevWant: Record<Dir, boolean> = { left: false, right: false, up: false, down: false };

    let timer: number | undefined;
    let lastTs = performance.now();

    const tick = () => {
        const now = performance.now();
        const dt = now - lastTs;
        lastTs = now;

        const pads = navigator.getGamepads?.() || [];
        const p = pads.find(Boolean);

        // Current desires
        let want: Record<Dir, boolean> = { left: false, right: false, up: false, down: false };

        // Current buttons (for rising edges on ABXY/LB/RB)
        let curButtons: boolean[] = prevButtons.slice();

        if (p) {
            curButtons = p.buttons.map(b => !!b?.pressed);

            // D-pad as buttons (standard mapping 12..15)
            want.up ||= !!p.buttons[12]?.pressed;
            want.down ||= !!p.buttons[13]?.pressed;
            want.left ||= !!p.buttons[14]?.pressed;
            want.right ||= !!p.buttons[15]?.pressed;

            // Some browsers/devices expose D-pad as a POV hat axis (commonly index 9, values -1..1)
            const hat = p.axes[9];
            if (typeof hat === "number") {
                // Typical mapping: -1 up, -0.714 up-right, -0.428 right, 0.0 neutral, etc.
                // We'll threshold into cardinal directions.
                const v = hat;
                if (v <= -0.5) want.up = true;
                if (v >= 0.5) want.down = true;
                // For left/right, many mappings use ±1/±0.428/±0.714 sequences around the circle.
                // Use axes[9] alone for up/down; we’ll still use stick for left/right below.
            }

            // Left stick axes
            const ax = p.axes[0] ?? 0;
            const ay = p.axes[1] ?? 0;
            if (ax < -dead) want.left = true;
            if (ax > dead) want.right = true;
            if (ay < -dead) want.up = true;
            if (ay > dead) want.down = true;

            // === Buttons with rising-edge detection ===
            const edge = (i: number) => curButtons[i] && !prevButtons[i];

            if (edge(0)) on("a");   // South
            if (edge(2)) on("y");   // West
            if (edge(3)) on("x");   // North
            if (edge(4)) on("lb");  // LB
            if (edge(5)) on("rb");  // RB
        }

        // === Directions: rising edge + hold/repeat ===
        for (const dir of DIRS) {
            const was = prevWant[dir];
            const is = want[dir];
            const h = hold[dir];

            if (is) {
                if (!h.down) {
                    // Rising edge (works even for very short taps now)
                    h.down = true;
                    h.heldMs = 0;
                    h.curRepeatMs = repeatStart;
                    h.nextFireMs = now + initialDelay;
                    on(dir);
                } else {
                    // Held: accelerate and fire repeats
                    h.heldMs += dt;

                    // Acceleration over time
                    const accelCut = (h.heldMs / 1000) * accelPerSec;

                    // Extra accel for strong stick tilt
                    let intensity = 0;
                    if (p) {
                        const ax = p.axes[0] ?? 0;
                        const ay = p.axes[1] ?? 0;
                        if (dir === "left") intensity = Math.max(0, (-ax - dead) / (1 - dead));
                        if (dir === "right") intensity = Math.max(0, (ax - dead) / (1 - dead));
                        if (dir === "up") intensity = Math.max(0, (-ay - dead) / (1 - dead));
                        if (dir === "down") intensity = Math.max(0, (ay - dead) / (1 - dead));
                    }
                    const intensityCut = intensity * axisBoost * 100;
                    const targetRepeat = Math.max(repeatMin, repeatStart - accelCut - intensityCut);
                    h.curRepeatMs = Math.max(targetRepeat, repeatMin);

                    if (now >= h.nextFireMs) {
                        on(dir);
                        h.nextFireMs = now + h.curRepeatMs;
                    }
                }
            } else {
                // Released (or not pressed this tick)
                h.down = false;
                h.heldMs = 0;
                h.nextFireMs = 0;
                h.curRepeatMs = repeatStart;
            }

            prevWant[dir] = is;
        }

        prevButtons = curButtons;
    };

    // Use setInterval for higher sampling, more reliable than rAF for short taps
    timer = window.setInterval(tick, intervalMs);
    return () => { if (timer) clearInterval(timer); };
}
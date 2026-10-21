import { ref, useTemplateRef } from "vue";

// A draggable divider between a top and a bottom area. The ratio (share of
// the height given to the top area) is remembered under `storageKey`.
//
// A "composable": a plain function that creates reactive state and functions
// for a component to use, so several components share the logic without
// sharing the state. Call it inside <script setup>.
//
// `containerRef`: the ref="..." name of the element holding both areas.
export function useSplitRatio(containerRef: string, storageKey: string, defaultRatio: number) {
  const MIN = 0.15;
  const MAX = 0.85;

  // useTemplateRef: the element with that ref="..." name in the template.
  const container = useTemplateRef<HTMLElement>(containerRef);
  const ratio = ref(read());

  // mousedown on the divider: follow the mouse until the button is released.
  function startResize(event: MouseEvent) {
    const box = container.value?.getBoundingClientRect();
    if (!box) {
      return;
    }
    event.preventDefault(); // don't start selecting text while dragging
    const onMove = (move: MouseEvent) => {
      const next = (move.clientY - box.top) / box.height;
      ratio.value = Math.min(MAX, Math.max(MIN, next));
    };
    const onUp = () => {
      window.removeEventListener("mousemove", onMove);
      try {
        localStorage.setItem(storageKey, String(ratio.value));
      } catch {
        // Not remembered; fine.
      }
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp, { once: true });
  }

  function read(): number {
    try {
      const value = Number(localStorage.getItem(storageKey));
      if (value >= MIN && value <= MAX) {
        return value;
      }
    } catch {
      // Use the default.
    }
    return defaultRatio;
  }

  return { ratio, startResize };
}

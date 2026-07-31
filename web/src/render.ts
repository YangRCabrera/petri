import { Universe } from './wasm/sim';

/** Sizes the page's `#sim-canvas` to the simulation grid and returns it. */
export function setupCanvas(width: number, height: number) {
  const canvas = document.querySelector<HTMLCanvasElement>('#sim-canvas')!;
  canvas.width = width;
  canvas.height = height;
  return canvas;
}

/**
 * Builds the per-frame render step: ticks the universe, then blits its
 * color buffer straight out of WASM linear memory onto the canvas.
 */
export function setupUniverseLoop(
  ctx: CanvasRenderingContext2D,
  universe: Universe,
  memory: WebAssembly.Memory,
  width: number,
  height: number,
) {
  const memorySize = width * height * 4; // RGBA bytes for the whole grid

  return function () {
    universe.tick();

    const cellsPtr = universe.get_ptr();
    const wasmMemoryArray = new Uint8ClampedArray(
      memory.buffer,
      cellsPtr,
      memorySize,
    );
    const imageData = new ImageData(wasmMemoryArray, width, height);
    ctx.putImageData(imageData, 0, 0);
  };
}

/** Frame-rate-limited animation loop with start/stop and pause/resume controls. */
export function setupLifetime(placeImage: () => void, fps: number) {
  let running = false;
  let paused = false;
  let animationId: ReturnType<typeof requestAnimationFrame>;
  let lastFrameTime: number; // timestamp (ms) of the last frame actually drawn
  const frameInterval = 1000 / fps; // ms budgeted per frame

  function isRunning() {
    return running;
  }
  function pause() {
    paused = true;
  }
  function resume() {
    paused = false;
  }
  function isPaused() {
    return paused;
  }
  function loop(time: number) {
    animationId = requestAnimationFrame(loop);

    const elapsed = time - lastFrameTime;
    if (elapsed < frameInterval) return;

    lastFrameTime = time - (elapsed % frameInterval);

    if (!isPaused()) placeImage();
  }
  function start() {
    running = true;
    lastFrameTime = performance.now();
    animationId = requestAnimationFrame(loop);
  }
  function stop() {
    running = false;
    cancelAnimationFrame(animationId);
  }

  return {
    start,
    stop,
    isRunning,
    pause,
    resume,
    isPaused,
  };
}

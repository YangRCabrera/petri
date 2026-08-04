import type { GrowthParams } from './params';
import { setupCanvas, setupLifetime, setupUniverseLoop } from './render';

export function buildSim(
  params: GrowthParams,
  WIDTH: number,
  HEIGHT: number,
  FPS: number,
  bindings: typeof import('./wasm/sim'),
  memory: WebAssembly.Memory,
) {
  const universe = bindings.Universe.new(
    WIDTH,
    HEIGHT,
    params.kernelRadius,
    params.ringWeights,
    params.growthTarget,
    params.growthWidth,
    params.timeStep,
  );
  universe.add_comet_blob(9, 0.1); // radius, heading (radians)

  const canvas = setupCanvas(WIDTH, HEIGHT);
  const ctx = canvas.getContext('2d')!;

  const placeImage = setupUniverseLoop(
    ctx,
    universe,
    memory,
    WIDTH,
    HEIGHT,
    FPS,
  );

  const lifetime = setupLifetime(placeImage, FPS);

  return { lifetime, universe };
}

import { applyParams, pushParams, type GrowthParams } from './params';
import { setupCanvas, setupLifetime, setupUniverseLoop } from './render';
import { SPECIES } from './species';

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
    params.growthFunction,
  );

  const firstSpecies = SPECIES[0];
  universe.load_pattern(
    firstSpecies.patternWidth,
    firstSpecies.patternHeight,
    firstSpecies.cells,
  );
  applyParams(firstSpecies.params);
  pushParams(firstSpecies.params, universe);

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

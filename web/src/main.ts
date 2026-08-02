import { setupCanvas, setupLifetime, setupUniverseLoop } from './render';
import './style.css';
import { getWasm } from './wasm-loader';

const { bindings, memory } = await getWasm();

const WIDTH = 300;
const HEIGHT = 300;

const FPS = 60;

const universe = bindings.Universe.new(
  WIDTH,
  HEIGHT,
  10, // kernel_radius
  new Float32Array([1.0]), // ring_weights: single flat ring
  0.15, // growth_target
  0.015, // growth_width
  0.1, // time_step
);
universe.add_comet_blob(9, 0.1); // radius, heading (radians)

const canvas = setupCanvas(WIDTH, HEIGHT);
const ctx = canvas.getContext('2d')!;

const placeImage = setupUniverseLoop(ctx, universe, memory, WIDTH, HEIGHT);

const lifetime = setupLifetime(placeImage, FPS);
lifetime.start();

window.addEventListener('keyup', (e) => {
  if (!lifetime.isRunning() || e.key !== 'Enter') return;
  if (lifetime.isPaused()) lifetime.resume();
  else lifetime.pause();
});

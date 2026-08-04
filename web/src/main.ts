import { buildSim } from './build-sim';
import { getWasm } from './wasm-loader';
import './style.css';
import {
  loadInitialParams,
  pushParams,
  readParams,
  setupParamsSync,
} from './params';

const { bindings, memory } = await getWasm();
const WIDTH = 300;
const HEIGHT = 300;
const FPS = 60;

const params = loadInitialParams();

const { lifetime, universe } = buildSim(
  params,
  WIDTH,
  HEIGHT,
  FPS,
  bindings,
  memory,
);

setupParamsSync(() => {
  const params = readParams();
  if (params) pushParams(params, universe);
});

lifetime.start();

window.addEventListener('keyup', (e) => {
  if (!lifetime.isRunning() || e.key !== 'Enter') return;
  if (lifetime.isPaused()) lifetime.resume();
  else lifetime.pause();
});

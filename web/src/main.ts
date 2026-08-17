import { buildSim } from './build-sim';
import { getWasm } from './wasm-loader';
import './style.css';
import {
  applyParams,
  loadInitialParams,
  pushParams,
  readParams,
  setupParamsSync,
} from './params';
import { SPECIES } from './species';
import { setupRleImport } from './rle-import';

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

const speciesSelect =
  document.querySelector<HTMLSelectElement>('#species-select')!;
for (const species of SPECIES) {
  const option = document.createElement('option');
  option.value = species.name;
  option.textContent = species.name;
  speciesSelect.appendChild(option);
}

speciesSelect.addEventListener('change', () => {
  const species = SPECIES.find((s) => s.name === speciesSelect.value)!;
  const params = species ? species.params : loadInitialParams();

  applyParams(params);
  pushParams(params, universe);

  universe.load_pattern(
    species.patternWidth,
    species.patternHeight,
    species.cells,
  );
});

setupRleImport(universe);

lifetime.start();

window.addEventListener('keyup', (e) => {
  if (!lifetime.isRunning() || e.key !== 'Enter') return;
  if (lifetime.isPaused()) lifetime.resume();
  else lifetime.pause();
});

import type { Universe } from './wasm/sim';

export interface GrowthParams {
  growthTarget: number;
  growthWidth: number;
  timeStep: number;
  kernelRadius: number;
  ringWeights: Float32Array;
}

const growthTargetInput =
  document.querySelector<HTMLInputElement>('#growth-target')!;
const growthWidthInput =
  document.querySelector<HTMLInputElement>('#growth-width')!;
const timeStepInput = document.querySelector<HTMLInputElement>('#time-step')!;
const kernelRadiusInput =
  document.querySelector<HTMLInputElement>('#kernel-radius')!;
const ringWeightsInput =
  document.querySelector<HTMLInputElement>('#ring-weights')!;
const paramsError =
  document.querySelector<HTMLParagraphElement>('#params-error')!;

const defaultParams: GrowthParams = {
  growthTarget: 0.15,
  growthWidth: 0.015,
  timeStep: 0.1,
  kernelRadius: 10,
  ringWeights: new Float32Array([1.0]),
};

export function loadInitialParams() {
  // TODO Load from URL
  return defaultParams;
}

export function pushParams(params: GrowthParams, universe: Universe) {
  universe.set_growth_target(params.growthTarget);
  universe.set_growth_width(params.growthWidth);
  universe.set_time_step(params.timeStep);
  universe.set_kernel_radius(params.kernelRadius);
  universe.set_ring_weights(params.ringWeights);
}

/** Parses a comma-separated list of ring weights, or null if any entry isn't a finite, non-negative number. */
function parseRingWeights(raw: string): Float32Array | null {
  const parts = raw.split(',').map((part) => part.trim());
  if (parts.length === 0 || parts.some((part) => part === '')) return null;

  const weights = parts.map(Number);
  if (weights.some((weight) => !Number.isFinite(weight) || weight < 0)) {
    return null;
  }

  return new Float32Array(weights);
}

/** Reads and validates the growth params off the form's current inputs. */
export function readParams(): GrowthParams | null {
  const growthTarget = growthTargetInput.valueAsNumber;
  const growthWidth = growthWidthInput.valueAsNumber;
  const timeStep = timeStepInput.valueAsNumber;
  const kernelRadius = kernelRadiusInput.valueAsNumber;
  const ringWeights = parseRingWeights(ringWeightsInput.value);

  if (!Number.isFinite(growthTarget) || growthTarget < 0 || growthTarget > 1) {
    paramsError.textContent = 'Growth target must be a number between 0 and 1.';
    return null;
  }
  if (!Number.isFinite(growthWidth) || growthWidth <= 0) {
    paramsError.textContent = 'Growth width must be a number greater than 0.';
    return null;
  }
  if (!Number.isFinite(timeStep) || timeStep <= 0) {
    paramsError.textContent = 'Time step must be a number greater than 0.';
    return null;
  }
  if (
    !Number.isFinite(kernelRadius) ||
    !Number.isInteger(kernelRadius) ||
    kernelRadius < 1
  ) {
    paramsError.textContent =
      'Kernel radius must be a whole number greater than 0.';
    return null;
  }
  if (!ringWeights) {
    paramsError.textContent =
      'Ring weights must be a comma-separated list of numbers >= 0.';
    return null;
  }

  paramsError.textContent = '';
  return { growthTarget, growthWidth, timeStep, kernelRadius, ringWeights };
}

/** Calls `onChange` on every edit to any params field, so a running universe can be kept live in sync. */
export function setupParamsSync(onChange: () => void) {
  for (const input of [
    growthTargetInput,
    growthWidthInput,
    timeStepInput,
    kernelRadiusInput,
    ringWeightsInput,
  ]) {
    input.addEventListener('input', onChange);
  }
}

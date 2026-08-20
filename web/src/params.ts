import { GrowthFunction, KernelFunction, type Universe } from './wasm/sim';

export interface GrowthParams {
  growthTarget: number;
  growthWidth: number;
  timeStep: number;
  kernelRadius: number;
  ringWeights: Float32Array;
  growthFunction: GrowthFunction;
  kernelFunction: KernelFunction;
}

const growthFunctionSelect =
  document.querySelector<HTMLSelectElement>('#growth-function')!;
const kernelFunctionSelect =
  document.querySelector<HTMLSelectElement>('#kernel-function')!;
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
  growthFunction: GrowthFunction.Gaussian,
  kernelFunction: KernelFunction.Exponential,
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
  universe.set_growth_function(params.growthFunction);
  universe.set_kernel_function(params.kernelFunction);
}

/** Writes `params` onto the form's inputs, the inverse of `readParams` — used when a species preset supplies its own kernel/growth params. */
export function applyParams(params: GrowthParams) {
  growthTargetInput.value = String(params.growthTarget);
  growthWidthInput.value = String(params.growthWidth);
  timeStepInput.value = String(params.timeStep);
  kernelRadiusInput.value = String(params.kernelRadius);
  ringWeightsInput.value = Array.from(params.ringWeights).join(', ');
  growthFunctionSelect.value = String(params.growthFunction);
  kernelFunctionSelect.value = String(params.kernelFunction);
  paramsError.textContent = '';
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
  const growthFunction = Number(growthFunctionSelect.value) as GrowthFunction;
  const kernelFunction = Number(kernelFunctionSelect.value) as KernelFunction;

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
  if (!Object.values(GrowthFunction).includes(growthFunction)) {
    paramsError.textContent = 'Growth function must be a recognized option.';
    return null;
  }
  if (!Object.values(KernelFunction).includes(kernelFunction)) {
    paramsError.textContent = 'Kernel function must be a recognized option.';
    return null;
  }

  paramsError.textContent = '';
  return {
    growthTarget,
    growthWidth,
    timeStep,
    kernelRadius,
    ringWeights,
    growthFunction,
    kernelFunction,
  };
}

/** Calls `onChange` on every edit to any params field, so a running universe can be kept live in sync. */
export function setupParamsSync(onChange: () => void) {
  for (const input of [
    growthFunctionSelect,
    kernelFunctionSelect,
    growthTargetInput,
    growthWidthInput,
    timeStepInput,
    kernelRadiusInput,
    ringWeightsInput,
  ]) {
    input.addEventListener('input', onChange);
  }
}

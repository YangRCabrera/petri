import init from './wasm/sim';

let wasmPromise: Promise<{
  bindings: typeof import('./wasm/sim');
  memory: WebAssembly.Memory;
}> | null = null;

export function getWasm() {
  if (!wasmPromise) {
    wasmPromise = init().then(async (wasmInstance) => {
      const bindings = await import('./wasm/sim');
      return {
        bindings,
        memory: wasmInstance.memory,
      };
    });
  }
  return wasmPromise;
}

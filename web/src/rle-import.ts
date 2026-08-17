import type { Universe } from './wasm/sim';

const rleInput = document.querySelector<HTMLTextAreaElement>('#rle-input')!;
const rleFile = document.querySelector<HTMLInputElement>('#rle-file')!;
const rleImportButton =
  document.querySelector<HTMLButtonElement>('#rle-import')!;
const rleError = document.querySelector<HTMLParagraphElement>('#rle-error')!;

/**
 * Wires the RLE import panel: picking a file reads its text into the
 * textarea, and the Import button hands the textarea's current contents to
 * `universe.load_rle`, surfacing any decode error (e.g. empty/unparseable
 * input) instead of silently doing nothing.
 */
export function setupRleImport(universe: Universe) {
  rleFile.addEventListener('change', async () => {
    const file = rleFile.files?.[0];
    if (!file) return;
    rleInput.value = await file.text();
  });

  rleImportButton.addEventListener('click', () => {
    try {
      universe.load_rle(rleInput.value);
      rleError.textContent = '';
    } catch (err) {
      rleError.textContent = String(err);
    }
  });
}

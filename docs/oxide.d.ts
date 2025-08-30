/* tslint:disable */
/* eslint-disable */
/**
 * WebAssembly start entry point for the runtime.
 *
 * Compiled and exported only on `wasm32` targets, this function is invoked
 * automatically by the `wasm-bindgen` bootstrap when the module is
 * instantiated. It installs a panic hook so Rust panics are logged to the
 * browser console, then delegates to [`run()`].
 *
 * Error propagation:
 * - Errors from [`run()`] are mapped into a `JsValue` and returned. This causes
 *   module instantiation to fail (e.g., the loader will observe a rejected
 *   Promise or thrown exception), allowing JavaScript to handle the failure.
 * - By default, the mapped value is a string. If your application needs a real
 *   `Error` object, adjust the mapper to return `js_sys::Error`.
 *
 * Returns:
 * - `Ok(())` on successful initialization and startup.
 * - `Err(JsValue)` if initialization fails; the value contains a formatted
 *   error message.
 *
 * This function is not meant to be called directly from JavaScript; it runs
 * once on module load.
 */
export function run_oxide_wasm(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly run_oxide_wasm: () => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_1: WebAssembly.Table;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_6: WebAssembly.Table;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h1906bbbc873e7667: (a: number, b: number) => void;
  readonly closure2898_externref_shim: (a: number, b: number, c: any) => void;
  readonly closure2900_externref_shim: (a: number, b: number, c: any, d: any) => void;
  readonly closure3072_externref_shim: (a: number, b: number, c: any) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;

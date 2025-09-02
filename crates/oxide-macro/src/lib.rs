use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn oxide_main(
        _attr: TokenStream,
        item: TokenStream,
) -> TokenStream
{
        let input_fn = parse_macro_input!(item as ItemFn);
        let fn_name = &input_fn.sig.ident;
        let fn_block = &input_fn.block;
        let fn_sig = &input_fn.sig;
        let fn_vis = &input_fn.vis;

        let expanded = quote! {
            // Original function
            #fn_vis #fn_sig {
                // Non-WASM initialization
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let _ = env_logger::builder().is_test(true).try_init();
                }

                // Original function body
                #fn_block
            }

            // WASM entrypoint
            #[cfg(target_arch = "wasm32")]
            #[wasm_bindgen::prelude::wasm_bindgen(start)]
            pub fn run_wasm() -> Result<(), wasm_bindgen::JsValue> {
                use wasm_bindgen::UnwrapThrowExt;

                console_error_panic_hook::set_once();

                console_log::init_with_level(log::Level::Info).unwrap_throw();

                #fn_name().map_err(|e| {
                    wasm_bindgen::JsValue::from_str(
                        &format!("Function `{}` failed: {e:#}", stringify!(#fn_name))
                    )
                })
            }
        };

        TokenStream::from(expanded)
}

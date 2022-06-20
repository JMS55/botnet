use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn bot(_: TokenStream, wrapped_function: TokenStream) -> TokenStream {
    let wrapped_function = parse_macro_input!(wrapped_function as ItemFn);
    let wrapped_function_name = &wrapped_function.sig.ident;

    quote! {
        #wrapped_function

        #[no_mangle]
        pub unsafe extern "C" fn __tick(
            bot_id: u64,
            bay_ptr: *const u8,
            bay_size: usize,
            network_memory_ptr: *mut u8,
            network_memory_size: usize,
        ) -> i32 {
            let bay_data = ::std::slice::from_raw_parts(bay_ptr, bay_size);
            let bay = ::botnet_api::rkyv::archived_value::<::botnet_api::Bay>(bay_data, 0);

            let network_memory = ::std::slice::from_raw_parts_mut(network_memory_ptr, network_memory_size);

            #wrapped_function_name(bot_id, bay, network_memory).into()
        }

        #[no_mangle]
        pub unsafe extern "C" fn __memalloc(size: usize) -> *mut u8 {
            let layout = ::std::alloc::Layout::from_size_align(size, ::std::mem::align_of::<usize>());
            match layout {
                Ok(layout) if layout.size() > 0 => ::std::alloc::alloc(layout),
                _ => ::std::ptr::null_mut(),
            }
        }
    }
    .into()
}

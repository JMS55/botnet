use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn};

#[proc_macro_attribute]
pub fn bot(network_memory_type: TokenStream, wrapped_function: TokenStream) -> TokenStream {
    let network_memory_type = parse_macro_input!(network_memory_type as Ident);

    let wrapped_function = parse_macro_input!(wrapped_function as ItemFn);
    let wrapped_function_name = &wrapped_function.sig.ident;

    quote! {
        use ::botnet_api::*;
        use ::botnet_api::pathfinding::*;

        #[allow(unused_must_use)]
        #wrapped_function

        #[no_mangle]
        pub unsafe extern "C" fn __tick(
            bot_id: u64,
            bay_pointer: *const u8,
            bay_size: usize,
            network_memory_pointer: *mut u8,
            network_memory_size: usize,
        ) {
            let bay_data = ::std::slice::from_raw_parts(bay_pointer, bay_size);
            let bay = ::botnet_api::rkyv::archived_root::<::botnet_api::Bay>(bay_data);

            let bot = bay.bots.get(&bot_id).unwrap();

            let network_memory = ::std::slice::from_raw_parts_mut(network_memory_pointer, network_memory_size);

            if network_memory[0] == 0 {
                network_memory[0] = 1;
                ::botnet_api::bincode::serialize_into(&mut network_memory[1..], &<#network_memory_type>::default()).unwrap();
            }
            let mut bot_network_memory: #network_memory_type = ::botnet_api::bincode::deserialize_from(&network_memory[1..]).unwrap();

            #wrapped_function_name(bot, bay, &mut bot_network_memory);

            ::botnet_api::bincode::serialize_into(&mut network_memory[1..], &mut bot_network_memory).unwrap();
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

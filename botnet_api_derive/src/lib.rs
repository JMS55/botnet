use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn};

/// Wrapper macro for a bot that handles unsafe setup boilerplate.
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

        /// Entry point for a bot tick that wraps neccesary setup work.
        #[no_mangle]
        pub unsafe extern "C" fn __tick(
            bot_id: u64,
            bay_pointer: *const u8,
            bay_size: usize,
            network_memory_pointer: *mut u8,
            network_memory_size: usize,
        ) {
            // Reconstruct bay
            let bay_data = ::std::slice::from_raw_parts(bay_pointer, bay_size);
            let bay = ::botnet_api::rkyv::archived_root::<::botnet_api::Bay>(bay_data);

            let bot = bay.bots.get(&bot_id).unwrap();

            // Reconstuct the raw network memory data
            let network_memory = ::std::slice::from_raw_parts_mut(network_memory_pointer, network_memory_size);

            // If network memory has never been written to, write a default instance of the network memory type.
            if network_memory[0] == 0 {
                network_memory[0] = 1;
                ::botnet_api::bincode::serialize_into(&mut network_memory[1..], &<#network_memory_type>::default()).unwrap();
            }

            // Deserialize network data into its type
            let mut bot_network_memory: #network_memory_type = ::botnet_api::bincode::deserialize_from(&network_memory[1..]).unwrap();

            // Call the wrapped tick function
            #wrapped_function_name(bot, bay, &mut bot_network_memory);

            // Serialize the network memory type back into raw memory
            ::botnet_api::bincode::serialize_into(&mut network_memory[1..], &mut bot_network_memory).unwrap();
        }

        /// Allocates a chunk of memory `size` bytes big, and returns a pointer to it.
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

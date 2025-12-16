extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Type};

#[proc_macro_derive(ScreenEngine)]
pub fn screen_engine_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let mut has_mouse = None;
    let mut has_keyboard = None;
    
    if let Data::Struct(data) = &input.data {
        for field in &data.fields {
            if let Some(ident) = &field.ident {
                if let Type::Path(type_path) = &field.ty {
                    if let Some(segment) = type_path.path.segments.last() {
                        let type_name = segment.ident.to_string();
                        match type_name.as_str() {
                            "MousePress" => has_mouse = Some(ident),
                            "Keyboard" => has_keyboard = Some(ident),
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    
    let mouse_reset = has_mouse.map(|field| {
        quote! {
            self.#field.just_pressed = false;
        }
    });
    
    let keyboard_reset = has_keyboard.map(|field| {
        quote! {
            self.#field.keys_just_pressed.clear();
        }
    });
    
    let expanded = quote! {
        impl ScreenEngine for #name {
            fn pixels(&self) -> &PixelsType {
                &self.pixels
            }
            
            fn reset_inputs(&mut self) {
                #mouse_reset
                #keyboard_reset
            }
        }
    };
    
    TokenStream::from(expanded)
}

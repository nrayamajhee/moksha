extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Obj)]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let d_i = parse_macro_input!(input as DeriveInput);
    let name = &d_i.ident;
    (quote! {
        use node::Node;
        impl Node for #name {
            fn storage(&self) -> RcRcell<Storage> {
                self.storage.clone()
            }
            fn obj_id(&self) -> Id {
                self.obj_id
            }
        }
        impl Obj for #name {}
    }).into()
}


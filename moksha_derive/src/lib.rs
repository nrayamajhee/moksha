extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Object)]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let d_i = parse_macro_input!(input as DeriveInput);
    let name = &d_i.ident;
    (quote! {
        impl Node for #name {
            fn storage(&self) -> RcRcell<Storage> {
                self.storage.clone()
            }
            fn obj_id(&self) -> Id {
                self.obj_id
            }
            fn update_id(&mut self, id: usize) {
                self.obj_id = id;
            }
            fn children(&self) -> NodeIterator<Self> {
                self.clone().into_iter()
            }
        }
        impl IntoIterator for #name {
            type Item = #name;
            type IntoIter = NodeIterator<Self::Item>;
            fn into_iter(self) -> Self::IntoIter {
                NodeIterator::<#name>::new(self)
            }
        }
        impl Object for #name {}
    }).into()
}


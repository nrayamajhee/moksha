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
            fn update_id(&mut self, id: usize) {
                self.obj_id = id;
            }
            fn children(&self) -> Iterator<Item = Self> {
                let storage = self.storage();
                let storage = storage.borrow();
                let children = Vec::new();
                storage.children(self.obj_id()).iter().map(|id| {
                    let child = self.clone();
                    child.update_id(*id);
                    child
                }).into_iter()
            }
        }
        impl Obj for #name {}
    }).into()
}


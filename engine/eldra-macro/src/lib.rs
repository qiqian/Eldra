use proc_macro::{Span, TokenStream};
use quote::quote;
use std::any::{type_name, TypeId};
use syn::{
    parse::ParseStream, parse_macro_input, Attribute, DeriveInput, Ident, Result,
};

#[proc_macro_derive(Reflection, attributes(attach))]
pub fn gen_reflection(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let attrs = &input.attrs;

    for attr in attrs {
        if attr.path().is_ident("attach") {

        }
    }

    TokenStream::from(quote! {
        impl IReflectable for #name {
            fn as_any(&self) -> &dyn Any { self }
            fn as_any_mut(&mut self) -> &mut dyn Any { self }
            fn real_type_id(&self) -> TypeId { TypeId::of::<Self>() }
        }
    })
}

#[proc_macro_derive(DropNotify, attributes(attach))]
pub fn gen_drop_notify(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let attrs = &input.attrs;

    TokenStream::from(quote! {
        impl Drop for #name {
            fn drop(&mut self) {
                engine_notify_drop_object(type_name::<#name>(), self.base.id);
            }
        }
    })
}


#[proc_macro_derive(ComponentAttr, attributes(attach))]
pub fn gen_component_attr(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let attrs = &input.attrs;

    TokenStream::from(quote! {
        impl IComponentAttr for #name {
            fn is_comp_uniq(&self) -> bool { Self::is_uniq() }
        }
    })
}

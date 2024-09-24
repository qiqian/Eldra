extern crate core;
use proc_macro::{TokenStream};
use quote::{*};
use syn::{*};

#[derive(Default,Clone)]
struct VarInfo {
    display : Option<Expr>,
    serialize : bool,
    readonly : bool,
}

#[proc_macro_derive(Reflection, attributes(display, serialize, readonly))]
pub fn gen_reflection(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let fields = if
        let syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
            ..}) = ast.data
    {
        named
    }
    else {
        panic!("You can derive only on a struct!")
    };

    // gather ReflectVarInfo
    let mut reflected = quote! {
        let mut v = std::vec::Vec::new();
    };
    for f in fields.iter() {
        let field_name = f.ident.clone().into_token_stream();
        let field_type = f.ty.clone().into_token_stream();
        println!("FIELD {field_name} {field_type}");
        let mut var = VarInfo::default();
        for attr in f.attrs.iter() {
            if attr.path().is_ident("display") {
                let display_name = attr.meta.require_name_value().unwrap().value.clone();
                var.display = Some(display_name);
            }
            else if attr.path().is_ident("serialize") {
                var.serialize = true;
            }
            else if attr.path().is_ident("readonly") {
                var.readonly = true;
            }
        }
        let serialize = var.serialize;
        if serialize {
            let readonly = var.readonly;
            let display_name = match var.display {
                Some(t) => {
                    quote! { Some(#t) }
                },
                None => quote! { None },
            };
            reflected.extend(quote! {
                            v.push(crate::reflection::ReflectVarInfo {
                                display : #display_name,
                                serialize : #serialize,
                                readonly : #readonly,
                                offset : std::mem::offset_of!(#name, #field_name) as u32,
                                size : std::mem::size_of::<#field_type>() as u32,
                            });
                    });
        }
    }
    reflected.extend(quote! { v });

    // generate Reflectable trait
    let gen_token = TokenStream::from(quote! {

        impl crate::reflection::Reflectable for #name {
            fn as_any(&self) -> &dyn Any { self }
            fn as_any_mut(&mut self) -> &mut dyn Any { self }
            fn real_type_id(&self) -> TypeId { TypeId::of::<Self>() }

            fn reflect_info(&self) -> std::vec::Vec<crate::reflection::ReflectVarInfo> {
                #reflected
            }
        }
    });

    // done
    let _gen_str = gen_token.clone().to_string();
    //println!("REFLECTION : {_gen_str}");
    gen_token
}

#[proc_macro_derive(DropNotify, attributes(attach))]
pub fn gen_drop_notify(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let _attrs = &input.attrs;

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
    let _attrs = &input.attrs;

    TokenStream::from(quote! {
        impl ComponentAttr for #name {
            fn is_comp_uniq(&self) -> bool { Self::is_uniq() }
        }
    })
}

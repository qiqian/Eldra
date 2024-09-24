extern crate core;
use proc_macro::{TokenStream};
use std::any::Any;
use quote::{*};
use syn::{*};

#[derive(Clone)]
struct VarInfo<'a> {
    display : Option<Expr>,
    serialize : bool,
    readonly : bool,
    field : &'a Field,
}

fn gen_reflect_info<'a>(struct_name: &Ident, vars: &Vec<VarInfo<'a>>) -> proc_macro2::TokenStream {
    let mut reflected = quote! {
        let mut v = std::vec::Vec::new();
    };
    for var in vars {
        let field_name = var.field.ident.clone().into_token_stream();
        let field_type = var.field.ty.clone().into_token_stream();
        let serialize = var.serialize;
        let readonly = var.readonly;
        let display_name = match var.display.clone() {
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
                                offset : std::mem::offset_of!(#struct_name, #field_name) as u32,
                                size : std::mem::size_of::<#field_type>() as u32,
                            });
                    });
    }
    reflected.extend(quote! { v });
    reflected
}


fn gen_yaml_serilizer<'a>(struct_name: &Ident, vars: &Vec<VarInfo<'a>>) -> proc_macro2::TokenStream {
    let mut reflected = quote! {};
    for var in vars {
        let field_tag = var.field.ident.clone().into_token_stream();
        let field_mark = format!("{{}}{} : \n", field_tag.to_string());
        reflected.extend(quote! {
            io.write(format!(#field_mark, indent.clone()).as_bytes());
        });
        let field_name = format!("{{}}field_name : \"{}\"\n", field_tag.to_string());
        let field_type = format!("{{}}field_type : \"{}\"\n",
             var.field.ty.clone().to_token_stream().to_string().replace(" ", ""));
        let readonly = format!("{{}}readonly : {}\n", var.readonly.to_string());
        reflected.extend(quote! {
            io.write(format!(#field_type, indent.clone() + "  ").as_bytes());
            io.write(format!(#readonly, indent.clone() + "  ").as_bytes());
            io.write(format!(#field_name, indent.clone() + "  ").as_bytes());
        });
        if var.display.is_some() {
            let d = format!("{{}}display_name : {}\n", var.display.clone().unwrap().to_token_stream().to_string());
            reflected.extend(quote! {
                io.write(format!(#d, indent.clone() + "  ").as_bytes());
            });
        }
        reflected.extend(quote! {
            io.write(format!("{}value : ", indent.clone() + "  ").as_bytes());
            if self.#field_tag.is_multi_line() {
                io.write("\n".as_bytes());
            }
            self.#field_tag.serialize_yaml(io, indent.clone() + "    ");
            io.write("\n".as_bytes());
        });
    }
    reflected
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
    let mut vars = vec!();
    for f in fields.iter() {
        let mut var = VarInfo { display: None, serialize: false, readonly: false, field:f };
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
            vars.push(var);
        }
    }

    let reflected = gen_reflect_info(name, &vars);
    let yaml_serializer = gen_yaml_serilizer(name, &vars);

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

        impl crate::reflection::Serializable for #name {
            fn is_multi_line(&self) -> bool { true }
            fn serialize_binary(&self, io: &mut dyn std::io::Write) {

            }
            fn deserialize_binary(&mut self, io: &mut dyn std::io::Read) {

            }
            fn serialize_yaml(&self, io: &mut dyn std::io::Write, indent: String) {
                #yaml_serializer
            }
            fn deserialize_yaml(&mut self, io: &mut dyn std::io::Read, indent: String) {

            }
        }
    });

    // done
    let _gen_str = gen_token.clone().to_string();
    println!("REFLECTION : {_gen_str}");
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

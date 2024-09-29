extern crate core;
use proc_macro::{TokenStream};
use quote::{*};
use syn::{*};

#[derive(Clone)]
struct VarInfo<'a> {
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
        reflected.extend(quote! {
                            v.push(crate::reflection::ReflectVarInfo {
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


fn gen_yaml_serilizer<'a>(vars: &Vec<VarInfo<'a>>) -> proc_macro2::TokenStream {
    let mut reflected = quote! {};
    for var in vars {
        let field_tag = var.field.ident.clone().into_token_stream();
        let field_mark = format!("{{}}{} : \n", field_tag.to_string());
        reflected.extend(quote! {
            let _ = io.write_all(format!(#field_mark, indent.clone()).as_bytes());
        });
        let field_type = format!("{{}}field_type : \"{}\"\n",
             var.field.ty.clone().to_token_stream().to_string().replace(" ", ""));
        reflected.extend(quote! {
            let _ = io.write_all(format!(#field_type, indent.clone() + "  ").as_bytes());
        });
        reflected.extend(quote! {
            let _ = io.write_all(format!("{}value : ", indent.clone() + "  ").as_bytes());
            if self.#field_tag.is_multi_line() {
                let _ = io.write_all("\n".as_bytes());
            }
            self.#field_tag.serialize_yaml(io, indent.clone() + "    ");
            let _ = io.write_all("\n".as_bytes());
        });
    }
    reflected
}

fn gen_yaml_deserilizer<'a>(vars: &Vec<VarInfo<'a>>) -> proc_macro2::TokenStream {
    let mut reflected = quote! {};
    for var in vars {
        let field_ident = var.field.ident.clone().into_token_stream();
        let field_name = field_ident.to_string();
        reflected.extend(quote! {
            {
                let field_data = &yaml[#field_name];
                if !field_data.is_null() && !field_data.is_badvalue() {
                    let field_value = &field_data["value"];
                    if !field_value.is_null() && !field_value.is_badvalue() {
                        self.#field_ident.deserialize_yaml(field_value);
                    }
                }
            }
        });
    }
    reflected
}

fn gen_binary_serilizer<'a>(vars: &Vec<VarInfo<'a>>) -> proc_macro2::TokenStream {
    let mut reflected = quote! {};
    for var in vars {
        let field_ident = var.field.ident.clone().into_token_stream();
        reflected.extend(quote! {
            let _ = self.#field_ident.serialize_binary(io);
        });
    }
    reflected
}

fn gen_binary_deserilizer<'a>(vars: &Vec<VarInfo<'a>>) -> proc_macro2::TokenStream {
    let mut reflected = quote! {};
    for var in vars {
        let field_ident = var.field.ident.clone().into_token_stream();
        reflected.extend(quote! {
            let _ = self.#field_ident.deserialize_binary(io);
        });
    }
    reflected
}


#[proc_macro_derive(Reflection, attributes(uuid, display, serialize, readonly))]
pub fn gen_reflection(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    // find uuid
    let mut uuid = None;
    for attr in ast.attrs.iter() {
        if attr.path().is_ident("uuid") {
            let uuid_str = attr.meta.require_name_value().unwrap().value.clone().to_token_stream();
            // println!("UUID {}", uuid_str);
            uuid = Some(uuid_str);
        }
    }
    let mut my_token = match uuid.clone() {
        Some(t) => quote!(
                impl #name {
                    pub fn type_uuid() -> Option<uuid::Uuid> {
                        match uuid::Uuid::from_str(#t) {
                            Ok(u) => Some(u),
                            Err(e) => None,
                        }
                    }
                }
            ),
        None => quote!(
            impl #name { pub fn type_uuid() -> Option<uuid::Uuid> { None } }
        ),
    };

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
    let mut has_serializable_fields = false;
    for f in fields.iter() {
        let mut var = VarInfo { serialize: false, readonly: false, field:f };
        for attr in f.attrs.iter() {
            if attr.path().is_ident("serialize") {
                var.serialize = true;
                has_serializable_fields = true;
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
    let yaml_serializer = gen_yaml_serilizer(&vars);
    let yaml_deerializer = gen_yaml_deserilizer(&vars);
    let binary_serializer = gen_binary_serilizer(&vars);
    let binary_deerializer = gen_binary_deserilizer(&vars);

    // generate Reflectable trait
    my_token.extend(quote! {
        impl crate::reflection::Reflectable for #name {
            fn as_any(&self) -> &dyn Any { self }
            fn as_any_mut(&mut self) -> &mut dyn Any { self }
            fn real_type_id(&self) -> TypeId { TypeId::of::<Self>() }
            fn reflect_info(&self) -> std::vec::Vec<crate::reflection::ReflectVarInfo> { #reflected }
        }
        impl crate::reflection::Serializable for #name {
            fn is_multi_line(&self) -> bool { #has_serializable_fields }
            fn get_type_uuid(&self) -> Option<uuid::Uuid> { #name::type_uuid() }
            fn serialize_binary(&self, io: &mut dyn std::io::Write) {
                #binary_serializer
            }
            fn deserialize_binary(&mut self, io: &mut dyn std::io::Read) {
                #binary_deerializer
            }
            fn serialize_yaml(&self, io: &mut dyn std::io::Write, indent: String) {
                #yaml_serializer
            }
            fn deserialize_yaml(&mut self, yaml: &yaml_rust2::Yaml) {
                #yaml_deerializer
            }
        }
    });

    // done
    let gen_token = TokenStream::from(my_token);
    let _gen_str = gen_token.clone().to_string();
    // println!("REFLECTION : {_gen_str}");
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
                engine_notify_drop_object(type_name::<#name>(), &self.instance_id);
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

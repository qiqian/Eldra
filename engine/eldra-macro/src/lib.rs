extern crate core;
use proc_macro::{TokenStream};
use std::any::{Any, TypeId};
use std::fmt::Pointer;
use std::ops::Deref;
use quote::{*};
use syn::{*};
use syn::punctuated::Punctuated;
use syn::token::Comma;

#[derive(Clone)]
struct VarInfo<'a> {
    display: proc_macro2::TokenStream,
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
        let display = var.display.clone();
        reflected.extend(quote! {
                            v.push(crate::reflection::ReflectVarInfo {
                                display: #display,
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
        let field_type = format!("{{}}field_type : \"{}\"",
             var.field.ty.clone().to_token_stream().to_string().replace(" ", ""));
        reflected.extend(quote! {
            let _ = io.write_all(format!(#field_type, indent.clone() + "  ").as_bytes());
            io.newline();
        });
        reflected.extend(quote! {
            let _ = io.write_all(format!("{}value : ", indent.clone() + "  ").as_bytes());
            if self.#field_tag.is_multi_line() {
                io.newline();
            }
            self.#field_tag.serialize_text(io, indent.clone() + "    ");
            io.newline();
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
                        self.#field_ident.deserialize_text(field_value);
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

fn gen_struct_reflection(fields: &Punctuated<Field, Comma>, ast: &DeriveInput) -> TokenStream {
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

    // gather ReflectVarInfo
    let mut vars = vec!();
    let mut has_serializable_fields = false;
    for f in fields.iter() {
        let mut var = VarInfo { display: quote! { "" }, serialize: false, readonly: false, field:f };
        for attr in f.attrs.iter() {
            if attr.path().is_ident("serialize") {
                var.serialize = true;
                has_serializable_fields = true;
            }
            else if attr.path().is_ident("readonly") {
                var.readonly = true;
            }
            else if attr.path().is_ident("display") {
                let display_name = attr.meta.require_name_value().unwrap().value.clone();
                var.display = quote! { #display_name };
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
    });
    if has_serializable_fields {
        my_token.extend(quote! {
            impl crate::reflection::Serializable for #name {
                fn is_multi_line(&self) -> bool { #has_serializable_fields }
                fn get_type_uuid(&self) -> Option<uuid::Uuid> { #name::type_uuid() }
                fn serialize_binary(&self, io: &mut dyn std::io::Write) {
                    #binary_serializer
                }
                fn deserialize_binary(&mut self, io: &mut dyn std::io::Read) {
                    #binary_deerializer
                }
                fn serialize_text(&self, io: &mut crate::reflection::SerializeTextWriter, indent: String) {
                    #yaml_serializer
                }
                fn deserialize_text(&mut self, yaml: &yaml_rust2::Yaml) {
                    #yaml_deerializer
                }
            }
        });
    }

    // done
    let gen_token = TokenStream::from(my_token);
    let _gen_str = gen_token.clone().to_string();
    // println!("REFLECTION : {_gen_str}");
    gen_token
}

fn extract_type_string(ty: &Type) -> String {
    let mut type_path: Vec<String> = match ty {
        syn::Type::Path(path) => {
            path.into_token_stream().into_iter().filter_map(|e| {
                let out = e.to_string();

                if out == "<" || out == ">" {
                    return None;
                } else {
                    return Some(out);
                }
            }).collect()
        },
        _ => panic!("Type '{}' has no path", ty.to_token_stream())
    };
    type_path.last().unwrap().clone()
}
fn gen_enum_reflection(variants: &Punctuated<Variant, Comma>, ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let mut gen_to_i32 = true;
    let mut to_i32 = quote! {};
    let mut to_string = quote! {};
    let mut yaml_serializer = quote! {};
    let mut yaml_deserializer = quote! {};
    let mut binary_serializer = quote! {};
    let mut binary_deserializer = quote! {};
    let mut index = 0u16;
    for v in variants.iter() {
        if !v.fields.is_empty() {
            gen_to_i32 = false;
            if v.fields.len() > 1 {
                panic!("Enum with more than 1 field can't generate reflection");
            }
        }
        let val_opt = v.discriminant.clone();
        if val_opt.is_none() {
            gen_to_i32 = false;
        }
        let id = &v.ident;
        let id_str = id.to_string();
        if gen_to_i32 {
            let val = &val_opt.unwrap().1;
            to_i32.extend(quote! {
                #name::#id => #val,
            });
        }
        if v.fields.is_empty() {
            // enum with no field
            to_string.extend(quote! {
                #name::#id => #id_str.to_string(),
            });
            binary_serializer.extend(quote! {
                #name::#id => { #index.serialize_binary(io); },
            });
            binary_deserializer.extend(quote! {
                #index => { *self = #name::#id; },
            });
            let yaml = format!("{{ enum: \"{}\" }}", id_str);
            yaml_serializer.extend(quote! {
                #name::#id => { #yaml.to_string().serialize_text(io, indent.clone()); },
            });
            yaml_deserializer.extend(quote! {
                #id_str => { *self = #name::#id; },
            });
        } else {
            let f;
            match v.fields {
                Fields::Unnamed(ref fields) => { f = fields.unnamed.iter().next().unwrap(); },
                _ => panic!("Enum with named field can't generate reflection")
            }
            let field_type = &f.ty.clone();
            // enum with 1 field
            to_string.extend(quote! {
                #name::#id(v) => #id_str.to_string() + "(" + &v.to_string() + ")",
            });
            binary_serializer.extend(quote! {
                #name::#id(v) => {
                    #index.serialize_binary(io);
                    v.serialize_binary(io);
                },
            });
            binary_deserializer.extend(quote! {
                #index => {
                    let mut v = #field_type ::default();
                    v.deserialize_binary(io);
                    *self = #name::#id(v);
                },
            });
            let field_type_str = extract_type_string(field_type);
            let yaml = if field_type_str == "String"
                { format!("{{{{ enum: \"{}\", val: \"{{}}\" }}}}", id_str) } else
                { format!("{{{{ enum: \"{}\", val: {{}} }}}}", id_str) };
            //println!("YAML {}/{:?}", field_type_str, yaml);
            yaml_serializer.extend(quote! {
                #name::#id(v) => { format!(#yaml, v).serialize_text(io, indent.clone()); },
            });
            yaml_deserializer.extend(quote! {
                #id_str => {
                    let mut v = #field_type ::default();
                    v.deserialize_text(&yaml["val"]);
                    *self = #name::#id(v);
                },
            });
        }
        index += 1;
    }
    let mut my_token = quote! {
        impl #name {
            fn to_string(&self) -> String {
                match self {
                    #to_string
                }
            }
        }
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.to_string())
            }
        }
        impl crate::reflection::Serializable for #name {
            fn is_multi_line(&self) -> bool { false }
            fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
            fn serialize_binary(&self, io: &mut dyn std::io::Write) {
                match self {
                    #binary_serializer
                }
            }
            fn deserialize_binary(&mut self, io: &mut dyn std::io::Read) {
                let mut val: u16 = 0;
                val.deserialize_binary(io);
                match val {
                    #binary_deserializer
                    _ => { panic!("invalid enum value, please regenerate binary data"); }
                }
            }
            fn serialize_text(&self, io: &mut crate::reflection::SerializeTextWriter, indent: String) {
                match self {
                    #yaml_serializer
                }
            }
            fn deserialize_text(&mut self, yaml: &yaml_rust2::Yaml) {
                let mut val = String::new();
                val.deserialize_text(&yaml["enum"]);
                match val.as_ref() {
                    #yaml_deserializer
                    _ => { panic!("invalid enum value, enum type changed ?"); }
                }
            }
        }
    };
    if gen_to_i32 {
        my_token.extend(quote! {
            impl #name {
                fn to_i32(&self) -> i32 {
                    match self {
                        #to_i32
                    }
                }
            }
        })
    }
    TokenStream::from(my_token)
}

#[proc_macro_derive(Reflection, attributes(uuid, display, serialize, readonly))]
pub fn gen_reflection(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    match ast.data {
        Data::Struct(ref s) => {
            match s.fields {
                Fields::Named(ref fields) => gen_struct_reflection(&fields.named, &ast),
                _ => panic!("You can derive Reflection only on a named-struct or int-enum!")
            }
        }
        Data::Enum(ref e) => gen_enum_reflection(&e.variants, &ast),
        Data::Union(_) => {
            panic!("You can derive Reflection only on a named-struct or int-enum!")
        }
    }
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

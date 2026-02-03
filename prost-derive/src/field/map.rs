use anyhow::{bail, Error};
use core::str::FromStr;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Expr, ExprLit, Ident, Lit, Meta, MetaNameValue, Token};

use crate::field::{scalar, set_option, tag_attr, Wrapper};

#[derive(Clone, Debug)]
pub enum MapTy {
    HashMap,
    BTreeMap,
}

impl MapTy {
    fn from_str(s: &str) -> Option<MapTy> {
        match s {
            "map" | "hash_map" => Some(MapTy::HashMap),
            "btree_map" => Some(MapTy::BTreeMap),
            _ => None,
        }
    }

    fn module(&self) -> Ident {
        match *self {
            MapTy::HashMap => Ident::new("hash_map", Span::call_site()),
            MapTy::BTreeMap => Ident::new("btree_map", Span::call_site()),
        }
    }

    fn lib(&self) -> TokenStream {
        match self {
            MapTy::HashMap => quote! { std },
            MapTy::BTreeMap => quote! { prost::alloc },
        }
    }

    fn raw_map_ref_method(&self) -> TokenStream {
        let method = match self {
            MapTy::HashMap => "raw_hash_map_key_ref",
            MapTy::BTreeMap => "raw_btree_map_key_ref",
        };
        TokenStream::from_str(method).unwrap()
    }

    fn raw_map_mut_ref_method(&self) -> TokenStream {
        let method = match self {
            MapTy::HashMap => "raw_hash_map_key_mut_ref",
            MapTy::BTreeMap => "raw_btree_map_key_mut_ref",
        };
        TokenStream::from_str(method).unwrap()
    }

    fn raw_map_value_ref_method(&self) -> TokenStream {
        let method = match self {
            MapTy::HashMap => "raw_hash_map_value_ref",
            MapTy::BTreeMap => "raw_btree_map_value_ref",
        };
        TokenStream::from_str(method).unwrap()
    }

    fn raw_map_value_mut_ref_method(&self) -> TokenStream {
        let method = match self {
            MapTy::HashMap => "raw_hash_map_value_mut_ref",
            MapTy::BTreeMap => "raw_btree_map_value_mut_ref",
        };
        TokenStream::from_str(method).unwrap()
    }

    fn raw_map_both_ref_method(&self) -> TokenStream {
        let method = match self {
            MapTy::HashMap => "raw_hash_map_ref",
            MapTy::BTreeMap => "raw_btree_map_ref",
        };
        TokenStream::from_str(method).unwrap()
    }

    fn raw_map_both_mut_ref_method(&self) -> TokenStream {
        let method = match self {
            MapTy::HashMap => "raw_hash_map_mut_ref",
            MapTy::BTreeMap => "raw_btree_map_mut_ref",
        };
        TokenStream::from_str(method).unwrap()
    }
}

fn fake_scalar(ty: scalar::Ty) -> scalar::Field {
    let kind = scalar::Kind::Plain(scalar::DefaultValue::new(&ty));
    scalar::Field {
        ty,
        kind,
        tag: 0, // Not used here
        wrapper: None,
    }
}

#[derive(Clone)]
pub struct Field {
    pub map_ty: MapTy,
    pub key_ty: scalar::Ty,
    pub value_ty: ValueTy,
    pub tag: u32,
    pub key_wrapper: Option<Wrapper>,
    pub value_wrapper: Option<TokenStream>,
}

impl Field {
    pub fn new(attrs: &[Meta], inferred_tag: Option<u32>) -> Result<Option<Field>, Error> {
        let mut types = None;
        let mut tag = None;
        let mut key_wrapper = None;

        for attr in attrs {
            if let Some(t) = tag_attr(attr)? {
                set_option(&mut tag, t, "duplicate tag attributes")?;
            } else if let Some(map_ty) = attr
                .path()
                .get_ident()
                .and_then(|i| MapTy::from_str(&i.to_string()))
            {
                let (k, v): (String, String) = match attr {
                    Meta::NameValue(MetaNameValue {
                        value:
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(lit), ..
                            }),
                        ..
                    }) => {
                        let items = lit.value();
                        let mut items = items.split(',').map(ToString::to_string);
                        let k = items.next().unwrap();
                        let v = match items.next() {
                            Some(k) => k,
                            None => bail!("invalid map attribute: must have key and value types"),
                        };
                        if items.next().is_some() {
                            bail!("invalid map attribute: {:?}", attr);
                        }
                        (k, v)
                    }
                    Meta::List(meta_list) => {
                        let nested = meta_list
                            .parse_args_with(Punctuated::<Ident, Token![,]>::parse_terminated)?
                            .into_iter()
                            .collect::<Vec<_>>();
                        if nested.len() != 2 {
                            bail!("invalid map attribute: must contain key and value types");
                        }
                        (nested[0].to_string(), nested[1].to_string())
                    }
                    _ => return Ok(None),
                };
                set_option(
                    &mut types,
                    (map_ty, key_ty_from_str(&k)?, ValueTy::from_str(&v)?),
                    "duplicate map type attribute",
                )?;
            } else if let Some(wrapper) = Wrapper::from_attr(attr)? {
                set_option(&mut key_wrapper, wrapper, "duplicate wrapper attribute")?;
            } else {
                return Ok(None);
            }
        }

        Ok(match (types, tag.or(inferred_tag)) {
            (Some((map_ty, key_ty, value_ty)), Some(tag)) => {
                let value_wrapper = key_wrapper.as_ref().and_then(|w| w.value_type_name.clone());
                Some(Field {
                    map_ty,
                    key_ty,
                    value_ty,
                    tag,
                    key_wrapper,
                    value_wrapper,
                })
            }
            _ => None,
        })
    }

    pub fn new_oneof(attrs: &[Meta]) -> Result<Option<Field>, Error> {
        Field::new(attrs, None)
    }

    /// Helper to wrap the map reference with key and/or value wrappers
    fn apply_wrappers(&self, ident: TokenStream, is_mut: bool) -> TokenStream {
        let has_key = self.key_wrapper.is_some();
        let has_value = self.value_wrapper.is_some();

        match (has_key, has_value) {
            (false, false) => {
                // No wrappers
                if is_mut {
                    quote! {&mut #ident}
                } else {
                    quote! {&#ident}
                }
            }
            (true, true) => {
                // Both key and value wrapped - use the "both" methods
                let key_type = &self.key_wrapper.as_ref().unwrap().type_name;
                let value_type = self.value_wrapper.as_ref().unwrap();
                let method = if is_mut {
                    self.map_ty.raw_map_both_mut_ref_method()
                } else {
                    self.map_ty.raw_map_both_ref_method()
                };
                if is_mut {
                    quote! {::prost::wrapper::#method::<#key_type, #value_type>(&mut #ident)}
                } else {
                    quote! {::prost::wrapper::#method::<#key_type, #value_type>(&#ident)}
                }
            }
            (true, false) => {
                // Only key wrapped
                let key_type = &self.key_wrapper.as_ref().unwrap().type_name;
                let method = if is_mut {
                    self.map_ty.raw_map_mut_ref_method()
                } else {
                    self.map_ty.raw_map_ref_method()
                };
                if is_mut {
                    quote! {::prost::wrapper::#method::<#key_type, _>(&mut #ident)}
                } else {
                    quote! {::prost::wrapper::#method::<#key_type, _>(&#ident)}
                }
            }
            (false, true) => {
                // Only value wrapped
                let value_type = self.value_wrapper.as_ref().unwrap();
                let method = if is_mut {
                    self.map_ty.raw_map_value_mut_ref_method()
                } else {
                    self.map_ty.raw_map_value_ref_method()
                };
                if is_mut {
                    quote! {::prost::wrapper::#method::<_, #value_type>(&mut #ident)}
                } else {
                    quote! {::prost::wrapper::#method::<_, #value_type>(&#ident)}
                }
            }
        }
    }

    /// Returns a statement which encodes the map field.
    pub fn encode(&self, ident: TokenStream) -> TokenStream {
        let tag = self.tag;
        let key_mod = self.key_ty.module();
        let ke = quote!(::prost::encoding::#key_mod::encode);
        let kl = quote!(::prost::encoding::#key_mod::encoded_len);
        let module = self.map_ty.module();
        let ident_value = self.apply_wrappers(ident, false);
        match &self.value_ty {
            ValueTy::Scalar(scalar::Ty::Enumeration(ty)) => {
                let default = quote!(#ty::default() as i32);
                quote! {
                    ::prost::encoding::#module::encode_with_default(
                        #ke,
                        #kl,
                        ::prost::encoding::int32::encode,
                        ::prost::encoding::int32::encoded_len,
                        &(#default),
                        #tag,
                        #ident_value,
                        buf,
                    );
                }
            }
            ValueTy::Scalar(value_ty) => {
                let val_mod = value_ty.module();
                let ve = quote!(::prost::encoding::#val_mod::encode);
                let vl = quote!(::prost::encoding::#val_mod::encoded_len);
                quote! {
                    ::prost::encoding::#module::encode(
                        #ke,
                        #kl,
                        #ve,
                        #vl,
                        #tag,
                        #ident_value,
                        buf,
                    );
                }
            }
            ValueTy::Message => quote! {
                ::prost::encoding::#module::encode(
                    #ke,
                    #kl,
                    ::prost::encoding::message::encode,
                    ::prost::encoding::message::encoded_len,
                    #tag,
                    #ident_value,
                    buf,
                );
            },
        }
    }

    /// Returns an expression which evaluates to the result of merging a decoded key value pair
    /// into the map.
    pub fn merge(&self, ident: TokenStream) -> TokenStream {
        let key_mod = self.key_ty.module();
        let km = quote!(::prost::encoding::#key_mod::merge);
        let module = self.map_ty.module();
        let ident_value = self.apply_wrappers(ident, true);
        match &self.value_ty {
            ValueTy::Scalar(scalar::Ty::Enumeration(ty)) => {
                let default = quote!(#ty::default() as i32);
                quote! {
                    ::prost::encoding::#module::merge_with_default(
                        #km,
                        ::prost::encoding::int32::merge,
                        #default,
                        #ident_value,
                        buf,
                        ctx,
                    )
                }
            }
            ValueTy::Scalar(value_ty) => {
                let val_mod = value_ty.module();
                let vm = quote!(::prost::encoding::#val_mod::merge);
                quote!(::prost::encoding::#module::merge(#km, #vm, #ident_value, buf, ctx))
            }
            ValueTy::Message => quote! {
                ::prost::encoding::#module::merge(
                    #km,
                    ::prost::encoding::message::merge,
                    #ident_value,
                    buf,
                    ctx,
                )
            },
        }
    }

    /// Returns an expression which evaluates to the encoded length of the map.
    pub fn encoded_len(&self, ident: TokenStream) -> TokenStream {
        let tag = self.tag;
        let key_mod = self.key_ty.module();
        let kl = quote!(::prost::encoding::#key_mod::encoded_len);
        let module = self.map_ty.module();
        let ident_value = self.apply_wrappers(ident, false);
        match &self.value_ty {
            ValueTy::Scalar(scalar::Ty::Enumeration(ty)) => {
                let default = quote!(#ty::default() as i32);
                quote! {
                    ::prost::encoding::#module::encoded_len_with_default(
                        #kl,
                        ::prost::encoding::int32::encoded_len,
                        &(#default),
                        #tag,
                        #ident_value,
                    )
                }
            }
            ValueTy::Scalar(value_ty) => {
                let val_mod = value_ty.module();
                let vl = quote!(::prost::encoding::#val_mod::encoded_len);
                quote!(::prost::encoding::#module::encoded_len(#kl, #vl, #tag, #ident_value))
            }
            ValueTy::Message => quote! {
                ::prost::encoding::#module::encoded_len(
                    #kl,
                    ::prost::encoding::message::encoded_len,
                    #tag,
                    #ident_value,
                )
            },
        }
    }

    pub fn clear(&self, ident: TokenStream) -> TokenStream {
        quote!(#ident.clear())
    }

    /// Returns methods to embed in the message.
    pub fn methods(&self, ident: &TokenStream) -> Option<TokenStream> {
        if let ValueTy::Scalar(scalar::Ty::Enumeration(ty)) = &self.value_ty {
            let key_ty = self.key_ty.rust_type();
            let key_ref_ty = self.key_ty.rust_ref_type();

            let get = Ident::new(&format!("get_{}", ident), Span::call_site());
            let insert = Ident::new(&format!("insert_{}", ident), Span::call_site());
            let take_ref = if self.key_ty.is_numeric() {
                quote!(&)
            } else {
                quote!()
            };

            let get_doc = format!(
                "Returns the enum value for the corresponding key in `{}`, \
                 or `None` if the entry does not exist or it is not a valid enum value.",
                ident,
            );
            let insert_doc = format!("Inserts a key value pair into `{}`.", ident);
            Some(quote! {
                #[doc=#get_doc]
                pub fn #get(&self, key: #key_ref_ty) -> ::core::option::Option<#ty> {
                    self.#ident.get(#take_ref key).cloned().and_then(|x| {
                        let result: ::core::result::Result<#ty, _> = ::core::convert::TryFrom::try_from(x);
                        result.ok()
                    })
                }
                #[doc=#insert_doc]
                pub fn #insert(&mut self, key: #key_ty, value: #ty) -> ::core::option::Option<#ty> {
                    self.#ident.insert(key, value as i32).and_then(|x| {
                        let result: ::core::result::Result<#ty, _> = ::core::convert::TryFrom::try_from(x);
                        result.ok()
                    })
                }
            })
        } else {
            None
        }
    }

    /// Returns a newtype wrapper around the map, implementing nicer Debug
    ///
    /// The Debug tries to convert any enumerations met into the variants if possible, instead of
    /// outputting the raw numbers.
    pub fn debug(&self, wrapper_name: TokenStream) -> TokenStream {
        let type_name = match self.map_ty {
            MapTy::HashMap => Ident::new("HashMap", Span::call_site()),
            MapTy::BTreeMap => Ident::new("BTreeMap", Span::call_site()),
        };

        // A fake field for generating the debug wrapper
        let key_wrapper = fake_scalar(self.key_ty.clone()).debug(quote!(KeyWrapper));
        let key = self.key_ty.rust_type();
        let key = if let Some(wrapper) = &self.key_wrapper {
            &wrapper.type_name
        } else {
            &key
        };
        let value_wrapper = self.value_ty.debug();
        let libname = self.map_ty.lib();
        let fmt = quote! {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                #key_wrapper
                #value_wrapper
                let mut builder = f.debug_map();
                for (k, v) in self.0 {
                    builder.entry(&KeyWrapper(k), &ValueWrapper(v));
                }
                builder.finish()
            }
        };
        match &self.value_ty {
            ValueTy::Scalar(ty) => {
                if let scalar::Ty::Bytes(_) = *ty {
                    return quote! {
                        struct #wrapper_name<'a>(&'a dyn ::core::fmt::Debug);
                        impl<'a> ::core::fmt::Debug for #wrapper_name<'a> {
                            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                                self.0.fmt(f)
                            }
                        }
                    };
                }

                let value = ty.rust_type();
                let value = if let Some(wrapper) = &self.value_wrapper {
                    wrapper
                } else {
                    &value
                };
                quote! {
                    struct #wrapper_name<'a>(&'a ::#libname::collections::#type_name<#key, #value>);
                    impl<'a> ::core::fmt::Debug for #wrapper_name<'a> {
                        #fmt
                    }
                }
            }
            ValueTy::Message => {
                if self.value_wrapper.is_some() {
                    let value_type = &self.value_wrapper;
                    quote! {
                        struct #wrapper_name<'a, V: 'a>(&'a ::#libname::collections::#type_name<#key, #value_type>);
                        impl<'a, V> ::core::fmt::Debug for #wrapper_name<'a, V>
                        where
                            V: ::core::fmt::Debug + 'a,
                        {
                            #fmt
                        }
                    }
                } else {
                    quote! {
                        struct #wrapper_name<'a, V: 'a>(&'a ::#libname::collections::#type_name<#key, V>);
                        impl<'a, V> ::core::fmt::Debug for #wrapper_name<'a, V>
                        where
                            V: ::core::fmt::Debug + 'a,
                        {
                            #fmt
                        }
                    }
                }
            }
        }
    }
}

fn key_ty_from_str(s: &str) -> Result<scalar::Ty, Error> {
    let ty = scalar::Ty::from_str(s)?;
    match ty {
        scalar::Ty::Int32
        | scalar::Ty::Int64
        | scalar::Ty::Uint32
        | scalar::Ty::Uint64
        | scalar::Ty::Sint32
        | scalar::Ty::Sint64
        | scalar::Ty::Fixed32
        | scalar::Ty::Fixed64
        | scalar::Ty::Sfixed32
        | scalar::Ty::Sfixed64
        | scalar::Ty::Bool
        | scalar::Ty::String => Ok(ty),
        _ => bail!("invalid map key type: {}", s),
    }
}

/// A map value type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueTy {
    Scalar(scalar::Ty),
    Message,
}

impl ValueTy {
    fn from_str(s: &str) -> Result<ValueTy, Error> {
        if let Ok(ty) = scalar::Ty::from_str(s) {
            Ok(ValueTy::Scalar(ty))
        } else if s.trim() == "message" {
            Ok(ValueTy::Message)
        } else {
            bail!("invalid map value type: {}", s);
        }
    }

    /// Returns a newtype wrapper around the ValueTy for nicer debug.
    ///
    /// If the contained value is enumeration, it tries to convert it to the variant. If not, it
    /// just forwards the implementation.
    fn debug(&self) -> TokenStream {
        match self {
            ValueTy::Scalar(ty) => fake_scalar(ty.clone()).debug(quote!(ValueWrapper)),
            ValueTy::Message => quote!(
                fn ValueWrapper<T>(v: T) -> T {
                    v
                }
            ),
        }
    }
}

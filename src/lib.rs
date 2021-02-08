use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Nothing, Parser},
    parse_macro_input,
    token::Mut,
    Block, Field, FieldsNamed, FnArg, Ident, ImplItemMethod, ItemStruct,
};

const TAP_THE_SIGN: &'static str =
    "`cached_property_struct` may only be used on structs with named fields";
const TAP_THE_SIGN_YOURSELF: &'static str =
    "`cached_property` may only be used on methods that only take `&mut self` as argument";
const STRUCT_PREFIX: &'static str = "CachedPropertyStorageFor";
const FIELD_NAME: &'static str = "cached_properties";
const METHOD_PREFIX: &'static str = "__cached_property_method_";
const PREFETCH_PREFIX: &'static str = "prefetch_";

#[proc_macro_attribute]
pub fn cached_property_struct(args: TokenStream, input: TokenStream) -> TokenStream {
    let names = parse_macro_input!(args as FieldsNamed);
    let mut ast = parse_macro_input!(input as ItemStruct);
    let name = &ast.ident;
    let cache_struct_name = STRUCT_PREFIX.to_string() + &name.to_string();
    let cache_struct_ident = Ident::new(&cache_struct_name, name.span());
    let cache_struct_base = (quote! {
        #[derive(Default)]
        struct #cache_struct_ident {
        }
    })
    .into();
    let mut cache_struct = parse_macro_input!(cache_struct_base as ItemStruct);
    if let syn::Fields::Named(ref mut fields) = &mut ast.fields {
        if let syn::Fields::Named(ref mut cache_fields) = &mut cache_struct.fields {
            for prop in names.named {
                let ident = prop.ident.expect(TAP_THE_SIGN);
                let ty = prop.ty;
                cache_fields.named.push(
                    Field::parse_named
                        .parse2(quote! { #ident : Option< #ty > })
                        .expect("failed to generate a cache field"),
                );
            }
        } else {
            panic!("cache struct didn't construct correctly oof");
        }
        let cache_field_ident = Ident::new(FIELD_NAME, name.span());
        fields.named.push(
            Field::parse_named
                .parse2(quote! { #cache_field_ident : #cache_struct_ident })
                .expect("failed to generate a field"),
        );
    } else {
        panic!("{}", TAP_THE_SIGN);
    }
    (quote! {
        #cache_struct
        #ast
    })
    .into()
}

#[proc_macro_attribute]
pub fn cached_property(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = parse_macro_input!(args as Nothing);
    let mut ast = parse_macro_input!(input as ImplItemMethod);
    let mut orig_sig = ast.sig.clone();
    let cache_ident = Ident::new(FIELD_NAME, orig_sig.ident.span());
    let storage_ident = &ast.sig.ident;
    let method_name = METHOD_PREFIX.to_string() + &ast.sig.ident.to_string();
    let method_ident = Ident::new(&method_name, orig_sig.ident.span());
    orig_sig.ident = method_ident.clone();
    let orig_block = ast.block.clone();

    let mut mut_ast = ast.clone();
    let mut mut_sig = &mut mut_ast.sig;
    let arg_count = mut_sig.inputs.len();
    match (mut_sig.inputs.first_mut(), arg_count) {
        (Some(FnArg::Receiver(rec)), 1) if rec.reference.is_some() => {
            if rec.mutability.is_none() {
                let moot_point = (quote! { mut }).into();
                rec.mutability = Some(parse_macro_input!(moot_point as Mut));
            }
        }
        _ => panic!("{}", TAP_THE_SIGN_YOURSELF),
    }
    let mut_name = PREFETCH_PREFIX.to_string() + &ast.sig.ident.to_string();
    mut_sig.ident = Ident::new(&mut_name, orig_sig.ident.span());
    let new_mut_block = (quote! {
        {
            match &self . #cache_ident . #storage_ident {
                Some(x) => x.clone(),
                None => {
                    let x = self . #method_ident ();
                    self . #cache_ident . #storage_ident = Some(x.clone());
                    x
                }
            }
        }

    })
    .into();
    mut_ast.block = parse_macro_input!(new_mut_block as Block);

    let new_immut_block = (quote! {
        {
            match &self . #cache_ident . #storage_ident {
                Some(x) => x.clone(),
                None => self . #method_ident (),
            }
        }

    })
    .into();
    ast.block = parse_macro_input!(new_immut_block as Block);

    (quote! {
        #orig_sig #orig_block
        #mut_ast
        #ast
    })
    .into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

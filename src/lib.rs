use proc_macro::TokenStream;
use quote::ToTokens;

#[proc_macro_attribute]
pub fn system(_attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let mut item = syn::parse::<syn::ItemFn>(tokens.into()).unwrap();

    let parameters = item.sig.inputs.iter().filter_map(|input| match input {
        syn::FnArg::Receiver(_) => None,
        syn::FnArg::Typed(pat_ty) => Some(&pat_ty.ty),
    });
    let check_params = quote::quote! { #(is_system_param::<#parameters>();)* };

    let block = syn::parse_quote! {{
        fn is_system_param<T: bevy::ecs::system::SystemParam>() {}
        #check_params
        panic!("#[bevycheck] should be removed after figuring out the error");
    }};
    item.block = Box::new(block);

    item.sig.inputs = syn::punctuated::Punctuated::new();
    item.into_token_stream().into()
}

#[cfg(test)]
mod tests;

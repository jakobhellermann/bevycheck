use proc_macro::TokenStream;
use proc_macro_error::{emit_error, proc_macro_error};
use quote::quote;
use syn::{__private::ToTokens, spanned::Spanned};

const VALID_BARE_TYPES: &[&str] = &[
    "Commands",
    "Res",
    "ResMut",
    "Query",
    "EventReader",
    "EventWriter",
];

#[proc_macro_attribute]
#[proc_macro_error]
pub fn system(_attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let item = syn::parse::<syn::ItemFn>(tokens).unwrap();
    let has_error = check_system_fn(&item.sig);

    if has_error {
        let name = &item.sig.ident;
        return (quote!(fn #name() {})).into();
    }

    item.into_token_stream().into()
}

fn check_system_fn(signature: &syn::Signature) -> bool {
    let mut has_error = false;

    for fn_arg in &signature.inputs {
        has_error |= check_system_fn_arg(fn_arg);
    }

    has_error
}

const ERR_MSG: &str = "invalid system parameter";

fn check_system_fn_arg(arg: &syn::FnArg) -> bool {
    let mut has_error = false;

    let ty = match arg {
        syn::FnArg::Receiver(receiver) => {
            emit_error!(receiver.span(), ERR_MSG);
            return true;
        }
        syn::FnArg::Typed(pat_type) => &*pat_type.ty,
    };
    match ty {
        syn::Type::Path(path) => {
            let last_segment = path.path.segments.last().unwrap();
            let name = last_segment.ident.to_string();

            if !VALID_BARE_TYPES.contains(&name.as_str()) {
                emit_error!(
                    ty.span(), ERR_MSG;
                    note = "cannot use type `{}` directly", name;
                    help = "to use it as a resource, use `Res<{}>` or `ResMut<{}>`", name, name
                );
                has_error = true;
            }
        }
        syn::Type::Reference(reference) => match &*reference.elem {
            syn::Type::Path(path) => {
                let last_segment = path.path.segments.last().unwrap();
                if last_segment.ident == "Commands" {
                    has_error = true;
                    emit_error!(ty.span(), ERR_MSG; help = "use `mut commands: Commands`");
                }
            }
            _ => todo!(),
        },
        _ => todo!(),
    }

    has_error
}

#[cfg(test)]
mod tests;

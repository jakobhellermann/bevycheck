use proc_macro::TokenStream;
use proc_macro_error::{abort, emit_error, proc_macro_error};
use quote::quote;
use syn::{__private::ToTokens, spanned::Spanned};

const VALID_BARE_TYPES: &[&str] = &[
    "Commands",
    "Res",
    "ResMut",
    "Local",
    "Query",
    "QuerySet",
    "DrawContext",
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

            match name.as_str() {
                "Query" => has_error |= check_query_generics(last_segment),
                other if VALID_BARE_TYPES.contains(&other) => {}
                _ => {
                    emit_error!(
                        ty.span(), ERR_MSG;
                        note = "cannot use type `{}` directly", name;
                        help = "to use it as a resource, use `Res<{}>` or `ResMut<{}>`", name, name
                    );
                    has_error = true;
                }
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

fn check_query_generics(path: &syn::PathSegment) -> bool {
    let mut has_error = false;

    let (query, filter) = match &path.arguments {
        syn::PathArguments::None | syn::PathArguments::Parenthesized(_) => {
            emit_error!(path.span(), ERR_MSG; note = "the query should have generic parameters");
            return true;
        }
        syn::PathArguments::AngleBracketed(args) => {
            let mut args = args.args.iter();
            let query = match args.next() {
                Some(syn::GenericArgument::Type(ty)) => ty,
                _ => {
                    emit_error!(path.span(), ERR_MSG; note = "the query should have generic type parameters");
                    return true;
                }
            };
            let filter = match args.next() {
                Some(syn::GenericArgument::Type(ty)) => Some(ty),
                None => None,
                _ => {
                    emit_error!(path.span(), ERR_MSG; note = "the query filter expects type parameters");
                    return true;
                }
            };
            (query, filter)
        }
    };

    has_error |= check_tuple_or_single(query, check_query_type);

    if let Some(filter) = filter {
        has_error |= check_tuple_or_single(filter, check_query_filter_type);
    }

    has_error
}

fn check_query_type(ty: &syn::Type) -> bool {
    const QUERY_ERROR_MSG: &str = "invalid query parameter";
    const VALID_QUERY_TYPES: &[&str] = &["Entity", "Flags"];

    match ty {
        syn::Type::Path(path) => {
            let last_segment = path.path.segments.last().unwrap();
            let name = last_segment.ident.to_string();

            match name.as_str() {
                "Option" => return check_query_type(option_type(last_segment)),
                other if VALID_QUERY_TYPES.contains(&other) => {}
                _ => {
                    emit_error!(
                        path.span(), QUERY_ERROR_MSG;
                        note = "`{}` is not a valid query type", name;
                        help = "if you want to query for a resource, use `&{}` or `&mut {}`", name, name
                    );
                    return true;
                }
            }
        }
        syn::Type::Reference(_) => {}
        ty => {
            emit_error!(ty.span(), QUERY_ERROR_MSG; note = "not a valid query parameter");
            return true;
        }
    }
    false
}

fn check_query_filter_type(ty: &syn::Type) -> bool {
    const QUERY_ERROR_MSG: &str = "invalid query filter";
    const VALID_QUERY_FILTER_TYPES: &[&str] = &[
        "Added",
        "Changed",
        "Mutated",
        "Or",
        "With",
        "WithBundle",
        "Without",
    ];

    match ty {
        syn::Type::Path(path) => {
            let last_segment = path.path.segments.last().unwrap();
            let name = last_segment.ident.to_string();
            if !VALID_QUERY_FILTER_TYPES.contains(&name.as_str()) {
                emit_error!(
                    path.span(), QUERY_ERROR_MSG;
                    note = "`{}` is not a valid query filter", name;
                    help = "if you want to check for {}'s existence, use `With<{}>``", name, name;
                );
                return true;
            }
        }
        _ => {}
    }
    false
}

fn option_type(last_segment: &syn::PathSegment) -> &syn::Type {
    let first_generic = match &last_segment.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            args.args.iter().next().and_then(|arg| match arg {
                syn::GenericArgument::Type(ty) => Some(ty),
                _ => None,
            })
        }
        _ => None,
    };
    match first_generic {
        Some(generic) => generic,
        None => abort!(
            last_segment.span(),
            "`Option` should have one generic argument"
        ),
    }
}
fn check_tuple_or_single<F: Fn(&syn::Type) -> bool>(ty: &syn::Type, f: F) -> bool {
    match ty {
        syn::Type::Tuple(tuple) => tuple.elems.iter().fold(false, |mut acc, item| {
            acc |= f(item);
            acc
        }),
        ty => f(ty),
    }
}

#[cfg(test)]
mod tests;

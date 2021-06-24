use proc_macro::TokenStream;
use proc_macro_error::{abort, emit_error, emit_warning, proc_macro_error};
use syn::{__private::ToTokens, spanned::Spanned};

#[proc_macro_attribute]
#[proc_macro_error]
pub fn system(_attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let mut item = syn::parse::<syn::ItemFn>(tokens).unwrap();
    let has_error = check_system_fn(&item.sig);

    let parameters = item.sig.inputs.iter().filter_map(|input| match input {
        syn::FnArg::Receiver(_) => None,
        syn::FnArg::Typed(pat_ty) => Some(&pat_ty.ty),
    });
    let check_params = (!has_error).then(|| {
        quote::quote! { #(is_system_param::<#parameters>();)* }
    });

    let block = syn::parse_quote! {
        {
            fn is_system_param<T: bevy::ecs::system::SystemParam>() {}
            #check_params
            panic!("#[bevycheck] should be removed after figuring out the error");
        }
    };

    item.block = Box::new(block);

    item.sig.inputs = syn::punctuated::Punctuated::new();
    item.into_token_stream().into()
}

fn check_system_fn(signature: &syn::Signature) -> bool {
    let mut has_error = false;

    let arg_count = signature.inputs.len();
    if arg_count >= 16 {
        emit_error!(signature.span(), "too many system parameters"; note = "only up to 16 parameters are supported"; help = "try bundling some parameters into tuples or a `#[derive(SystemParam)]` struct");
        return true;
    }

    for fn_arg in &signature.inputs {
        has_error |= check_system_fn_arg(fn_arg);
    }

    has_error
}

const ERR_MSG: &str = "invalid system parameter";

fn check_system_fn_arg(arg: &syn::FnArg) -> bool {
    match arg {
        syn::FnArg::Receiver(receiver) => {
            emit_error!(receiver.span(), ERR_MSG);
            true
        }
        syn::FnArg::Typed(pat_type) => check_system_param_ty(Some(&*pat_type.pat), &*pat_type.ty),
    }
}

fn check_system_param_ty(pat: Option<&syn::Pat>, ty: &syn::Type) -> bool {
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
        "In,",
    ];

    match ty {
        syn::Type::Path(path) => {
            let last_segment = path.path.segments.last().unwrap();
            let name = last_segment.ident.to_string();

            match name.as_str() {
                "Query" => check_query_generics(last_segment),
                "QuerySet" => check_query_set_generics(last_segment),
                other if VALID_BARE_TYPES.contains(&other) => false,
                _ => {
                    emit_warning!(
                        ty.span(), "possibly invalid system parameter";
                        note = "bevycheck can't figure out whether `{}` is a valid system param", name;
                        help = "to use it as a resource, use `Res<{}>` or `ResMut<{}>`", name, name
                    );
                    false
                }
            }
        }
        syn::Type::Reference(reference) => {
            if let syn::Type::Path(path) = &*reference.elem {
                let last_segment = path.path.segments.last().unwrap();
                if last_segment.ident == "Commands" {
                    emit_error!(ty.span(), ERR_MSG; help = "use `mut commands: Commands`");
                    return true;
                }
                if last_segment.ident == "Assets" {
                    let generic_ty = first_generic(last_segment).into_token_stream().to_string();
                    let pat = match pat {
                        Some(pat) => pat.into_token_stream().to_string(),
                        None => "assets".to_string(),
                    };
                    emit_error!(ty.span(), ERR_MSG; help = "use `mut {}: ResMut<Assets<{}>>`", pat, generic_ty);
                    return true;
                }
            }

            emit_warning!(ty.span(), "possibly invalid system parameter"; note = "bevycheck can't figure out whether this is a valid system param");
            false
        }
        syn::Type::Tuple(tuple) => tuple.elems.iter().fold(false, |mut acc, ty| {
            acc |= check_system_param_ty(None, ty);
            acc
        }),
        _ => {
            emit_warning!(ty.span(), "possibly invalid system parameter"; note = "bevycheck can't figure out whether this is a valid system param");
            false
        }
    }
}

fn check_query_set_generics(path: &syn::PathSegment) -> bool {
    let queries = match first_generic(path) {
        syn::Type::Tuple(tuple) => tuple.elems.iter(),
        _ => {
            emit_error!(path.span(), "invalid QuerySet"; note = "the first parameter of `QuerySet` should be a tuple of queries");
            return true;
        }
    };

    queries
        .map(|ty| match type_with_name(ty, "Query") {
            Some(query) => check_query_generics(query),
            None => {
                emit_error!(path.span(), "invalid QuerySet"; note = "the first parameter of `QuerySet` should be a tuple of queries");
                true
            }
        })
        .fold(false, |mut acc, item| {
            acc |= item;
            acc
        })
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
            let query_type = args.find_map(|arg| match arg {
                syn::GenericArgument::Type(ty) => Some(ty),
                _ => None,
            });
            let query_type = match query_type {
                Some(ty) => ty,
                None => {
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
            (query_type, filter)
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
                "Option" => return check_query_type(first_generic(last_segment)),
                other if VALID_QUERY_TYPES.contains(&other) => {}
                _ => {
                    emit_error!(
                        path.span(), QUERY_ERROR_MSG;
                        note = "`{}` is not a valid query type", name;
                        help = "if you want to query for a component, use `&{}` or `&mut {}`", name, name
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
    // types like Added<T>, Changed<T> which filter based on a generic component
    const VALID_COMPONENT_QUERY_FILTER_TYPES: &[&str] = &[
        "Added",
        "Changed",
        "Mutated",
        "With",
        "WithBundle",
        "Without",
    ];

    match ty {
        syn::Type::Path(path) => {
            let last_segment = path.path.segments.last().unwrap();
            let name = last_segment.ident.to_string();

            if VALID_COMPONENT_QUERY_FILTER_TYPES.contains(&name.as_str()) {
                let generic = first_generic(last_segment);
                let inner_name = match named_type(generic) {
                    Some(component) => component.ident.to_string(),
                    None => {
                        emit_error!(
                            path.span(), QUERY_ERROR_MSG;
                            note = "`{}` should be used like `{}<Component>", name, name;
                        );
                        return true;
                    }
                };
                if VALID_COMPONENT_QUERY_FILTER_TYPES.contains(&inner_name.as_str()) {
                    emit_error!(
                        path.span(), QUERY_ERROR_MSG;
                        note = "`{}` should be used like `{}<Component>", name, name;
                    );
                    return true;
                }
            } else if name == "Or" {
                let generic = first_generic(last_segment);
                let tuple = match generic {
                    syn::Type::Tuple(tuple) => tuple,
                    _ => {
                        emit_error!(
                            generic.span(), QUERY_ERROR_MSG;
                            note = "`Or` filters expect a tuple of query filters as their generic argument",
                        );
                        return true;
                    }
                };
                return check_tuple(tuple, check_query_filter_type);
            } else {
                emit_error!(
                    path.span(), "unknown query filter";
                    note = "`{}` is not a known query filter", name;
                    help = "if you want to check for {}'s existence, use `With<{}>``", name, name;
                );
                return true;
            }
        }
        syn::Type::Reference(reference) => {
            let name = named_type(&reference.elem).map(|path| path.ident.to_string());
            let (name, type_name) = match &name {
                Some(name) => (name.as_str(), name.as_str()),
                None => ("a component", "Component"),
            };

            emit_error!(
                reference.span(), QUERY_ERROR_MSG;
                help = "if you want to check for {}'s existence, use `With<{}>``", name, type_name;
            );
            return true;
        }
        _ => {}
    }
    false
}

fn first_generic(last_segment: &syn::PathSegment) -> &syn::Type {
    let first_generic = match &last_segment.arguments {
        syn::PathArguments::AngleBracketed(args) => args.args.iter().find_map(|arg| match arg {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        }),
        _ => None,
    };
    match first_generic {
        Some(generic) => generic,
        None => abort!(last_segment.span(), "should have one generic argument"),
    }
}

fn named_type<'a>(ty: &'a syn::Type) -> Option<&'a syn::PathSegment> {
    match ty {
        syn::Type::Path(path) => {
            let last_segment = path.path.segments.last().unwrap();
            Some(last_segment)
        }
        _ => None,
    }
}
fn type_with_name<'a>(ty: &'a syn::Type, name: &str) -> Option<&'a syn::PathSegment> {
    named_type(ty).filter(|segment| segment.ident == name)
}

fn check_tuple_or_single<F: Fn(&syn::Type) -> bool>(ty: &syn::Type, f: F) -> bool {
    match ty {
        syn::Type::Tuple(tuple) => check_tuple(tuple, f),
        ty => f(ty),
    }
}
fn check_tuple<F: Fn(&syn::Type) -> bool>(tuple: &syn::TypeTuple, f: F) -> bool {
    tuple.elems.iter().fold(false, |acc, item| acc || f(item))
}

#[cfg(test)]
mod tests;

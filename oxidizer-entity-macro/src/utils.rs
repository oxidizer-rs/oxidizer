use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    spanned::Spanned, AngleBracketedGenericArguments, Field, GenericArgument, Meta, Path,
    PathArguments, PathSegment, Type, TypePath,
};

pub fn iterate_angle_bracketed(
    ab: &AngleBracketedGenericArguments,
    expected: &Vec<String>,
    index: usize,
) -> bool {
    let index = index;

    if expected.len() == index {
        return true;
    }

    for arg in &ab.args {
        let res = match arg {
            GenericArgument::Type(Type::Path(tp)) => check_type_order(tp, expected, index),
            _ => unimplemented!(),
        };

        if res {
            return true;
        }
    }

    false
}

pub fn iterate_path_arguments(seg: &PathSegment, expected: &Vec<String>, index: usize) -> bool {
    let mut index = index;

    if expected.len() == index {
        return true;
    }

    if seg.ident.to_string() == expected[index] {
        index += 1;
    }

    if expected.len() == index {
        return true;
    }

    match &seg.arguments {
        PathArguments::AngleBracketed(angle) => iterate_angle_bracketed(angle, expected, index),
        PathArguments::Parenthesized(_paren) => unimplemented!(),
        PathArguments::None => expected.len() == index,
    }
}

pub fn iterate_path_segments(p: &Path, expected: &Vec<String>, index: usize) -> bool {
    let index = index;

    if expected.len() == index {
        return true;
    }

    for seg in p.segments.iter() {
        if iterate_path_arguments(seg, &expected, index) {
            return true;
        }
    }

    expected.len() == index
}

pub fn check_type_order(p: &TypePath, expected: &Vec<String>, index: usize) -> bool {
    let mut index = index;

    if expected.len() == index {
        return true;
    }

    if let Some(ident) = p.path.get_ident() {
        if ident.to_string() == expected[0] {
            index += 1;
        }
    }

    iterate_path_segments(&p.path, expected, index)
}

pub fn is_typed_with(segment: &PathSegment, expected: Vec<&str>) -> bool {
    let expected = expected.iter().map(|v| v.to_string()).collect();
    iterate_path_arguments(segment, &expected, 0)
}

pub fn is_chrono_option(segment: &PathSegment) -> bool {
    let expected: Vec<&str> = vec!["Option", "DateTime", "Utc"];
    let no_option_expected: Vec<&str> = vec!["DateTime", "Utc"];

    is_typed_with(segment, expected) || is_typed_with(segment, no_option_expected)
}

pub fn search_attr_in_field(field: &Field, attr: &str) -> bool {
    for option in (&field.attrs).into_iter() {
        let option = option.parse_meta().unwrap();
        match option {
            Meta::Path(path) if path.get_ident().unwrap().to_string() == attr => {
                return true;
            }
            _ => {}
        }
    }
    return false;
}

/// is_integer_type returns tuple indicating whether the type is an integer type and the type
/// itself cast to string
pub fn is_integer_type(ty: &Type) -> (bool, &str) {
    let segments = match ty {
        syn::Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => segments,
        _ => unimplemented!(),
    };
    match segments.first().unwrap() {
        PathSegment { ident, .. } if ident.to_string() == "i8" => (true, "i8"),
        PathSegment { ident, .. } if ident.to_string() == "i16" => (true, "i16"),
        PathSegment { ident, .. } if ident.to_string() == "i32" => (true, "i32"),
        PathSegment { ident, .. } if ident.to_string() == "i64" => (true, "i64"),
        PathSegment { ident, .. } if ident.to_string() == "u32" => (true, "u32"),
        PathSegment { ident, .. } if ident.to_string() == "u64" => (true, "u64"),
        _ => (false, ""),
    }
}

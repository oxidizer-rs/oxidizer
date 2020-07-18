use syn::{Path, Type, TypePath, PathSegment, PathArguments, AngleBracketedGenericArguments, GenericArgument};

pub fn iterate_angle_bracketed(ab: &AngleBracketedGenericArguments, expected: &Vec<String>, index: usize) -> bool {
    let mut index = index;

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
        PathArguments::AngleBracketed(angle) => {
            iterate_angle_bracketed(angle, expected, index)
        },
        PathArguments::Parenthesized(_paren) => unimplemented!(),
        PathArguments::None => expected.len() == index,
    }
}

pub fn iterate_path_segments(p: &Path, expected: &Vec<String>, index: usize) -> bool {
    let mut index = index;

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
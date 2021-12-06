use std::str::FromStr;

use diesel::pg::Pg;
use diesel::row::NamedRow;

pub fn convert_option<T, R: From<T>>(opt: Option<T>) -> Option<R> {
    match opt {
        Some(unwrapped) => Some(R::from(unwrapped)),
        None => None,
    }
}

pub fn convert_vec<T, R: From<T>>(vec: Vec<T>) -> Vec<R> {
    let mut new_vec = Vec::<R>::new();
    for elem in vec {
        new_vec.push(R::from(elem));
    }

    new_vec
}

pub fn vec_u8_to_string(vec: Vec<u8>) -> String {
    let mut vec_str = String::new();
    for elem in &vec {
        vec_str = format!("{}{:x}", vec_str, elem);
    }

    vec_str
}

pub fn option_vec_u8_to_string(vec: Option<Vec<u8>>) -> String {
    match vec {
        Some(vec) => vec_u8_to_string(vec),
        None => String::new(),
    }
}

pub fn vec_u64_to_vec_string(vec: Vec<u64>) -> Vec<String> {
    let mut epoch_index_str = Vec::<String>::new();
    for epoch_index in vec {
        epoch_index_str.push(u64::to_string(&epoch_index));
    }

    epoch_index_str
}

pub fn convert_row_string_to_u64<R: NamedRow<Pg>>(
    row: &R,
    row_name: &str,
) -> diesel::deserialize::Result<u64> {
    let val_str: String = row.get(row_name)?;
    Ok(u64::from_str(&val_str).unwrap())
}

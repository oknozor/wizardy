use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, Ident};

pub fn get_field_idents(fields: Vec<Field>) -> Vec<Ident> {
    fields
        .iter()
        .cloned()
        .map(|field| field.ident.unwrap())
        .collect()
}

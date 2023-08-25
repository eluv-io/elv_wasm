extern crate meta;

use std::collections::HashMap;

#[test]
fn test_get_meta_impl() {
    let qp = HashMap::<String, Vec<String>>::new();
    let md = HashMap::<String, serde_json::Value>::new();
    meta::do_get_meta_impl(&qp, md).unwrap();
}

#[test]
fn test_set_meta_impl() {
    let qp = HashMap::<String, Vec<String>>::new();
    let md = HashMap::<String, serde_json::Value>::new();
    meta::do_set_meta_impl(&qp, md).unwrap();
}

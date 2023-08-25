extern crate elvwasm;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

use elvwasm::{implement_bitcode_module, jpc, make_success_json, register_handler};
use serde_json::json;

implement_bitcode_module!("get_meta", do_get_meta, "set_meta", do_set_meta);

pub fn do_get_meta_impl(
    qp: &HashMap<String, Vec<String>>,
    md: HashMap<String, serde_json::Value>,
) -> CallResult {
    Ok(vec![])
}

#[no_mangle]
fn do_get_meta(bcc: &mut elvwasm::BitcodeContext) -> CallResult {
    let http_p = &bcc.request.params.http;
    let qp = &http_p.query;
    let meta: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&bcc.sqmd_get_json_resolve(&http_p.path)?)
            .into_iter()
            .collect::<serde_json::Map<String, serde_json::Value>>()
            .into_iter()
            .collect();
    let id = &bcc.request.id;
    let meta_return = do_get_meta_impl(qp, meta)?;
    bcc.write_stream("fos", &meta_return)?;
    bcc.callback(200, "application/json", meta_return.len())?;
    make_success_json(
        &json!(
        {
            "headers" : "application/json",
            "body" : "SUCCESS",
            "result" : "complete",
        }),
        id,
    )
}

pub fn do_set_meta_impl(
    qp: &HashMap<String, Vec<String>>,
    md: HashMap<String, serde_json::Value>,
) -> CallResult {
    Ok(vec![])
}

#[no_mangle]
fn do_set_meta(bcc: &mut elvwasm::BitcodeContext) -> CallResult {
    let http_p = &bcc.request.params.http;
    let qp = &http_p.query;
    let meta: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&bcc.sqmd_get_json_resolve(&http_p.path)?)
            .into_iter()
            .collect::<serde_json::Map<String, serde_json::Value>>()
            .into_iter()
            .collect();
    let id = &bcc.request.id;
    let meta_return = do_set_meta_impl(qp, meta)?;
    bcc.write_stream("fos", &meta_return)?;
    bcc.callback(200, "application/json", meta_return.len())?;
    make_success_json(
        &json!(
        {
            "headers" : "application/json",
            "body" : "SUCCESS",
            "result" : "complete",
        }),
        id,
    )
}

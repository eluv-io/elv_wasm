extern crate elvwasm;
extern crate serde_json;
extern crate serde;

use elvwasm::{implement_bitcode_module, jpc, register_handler, make_success_json};
use serde_json::json;

implement_bitcode_module!("playout_selector", do_playout_selector);


/// .
///
/// # Errors
///
/// This function will return an error if .
#[no_mangle]
fn do_playout_selector(bcc: &mut elvwasm::BitcodeContext<>) -> CallResult {
    let http_p = &bcc.request.params.http;
    let _qp = &http_p.query;
    let id = &bcc.request.id;
    make_success_json(&json!(
        {
            "headers" : "application/json",
            "body" : "SUCCESS",
            "result" : "complete",
        }), id)
}
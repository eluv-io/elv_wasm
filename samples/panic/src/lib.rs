extern crate elvwasm;
extern crate serde;
extern crate serde_json;

use elvwasm::{
    implement_bitcode_module, jpc, make_success_json, register_handler,
};
use serde_json::json;

implement_bitcode_module!("panic", do_panic);

/// A bitcode example triggering a panic through division by zero
#[no_mangle]
fn do_panic(bcc: &mut elvwasm::BitcodeContext) -> CallResult {
    let mut divisor = 0;
    if bcc.request.id.is_empty() {
        divisor = 1
    }
    let _div0 = 1/divisor;
    let id = &bcc.request.id;
    make_success_json(
        &json!(
        {
            "headers" : "application/json",
            "result" : "complete",
        }),
        id,
    )
}

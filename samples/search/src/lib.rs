extern crate elvwasm;
extern crate serde_json;
#[macro_use(defer)] extern crate scopeguard;
use serde_json::json;
use serde::{Deserialize, Serialize};


use elvwasm::{implement_bitcode_module, jpc, register_handler, BitcodeContext, ErrorKinds};

implement_bitcode_module!("crawl", do_crawl);

fn do_crawl<>(bcc: &mut elvwasm::BitcodeContext<>) -> CallResult {
    let http_p = &bcc.request.params.http;
    let qp = &http_p.query;
    BitcodeContext::log(&format!("In do_crawl hash={} headers={:#?} query params={:#?}",&bcc.request.q_info.hash, &http_p.headers, qp));
    let id = &bcc.request.id;
    bcc.make_success_json(&json!(
        {
            "headers" : "application/json",
            "body" : "SUCCESS",
            "result" : 0,
        }), id)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

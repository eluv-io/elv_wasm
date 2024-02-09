extern crate elvwasm;
extern crate serde_json;
use serde_json::json;

use elvwasm::{implement_bitcode_module, jpc, register_handler, ErrorKinds};

implement_bitcode_module!("proxy", do_proxy);

use std::collections::HashMap;

struct Replacer {
    missing: HashMap<String, Vec<String>>,
}

impl Replacer {
    fn replace_all(
        &mut self,
        buf: &mut String,
        replacements: &HashMap<String, Vec<String>>,
    ) -> String {
        for (key, value) in replacements {
            let real_key = format!("${{{}}}", key);
            self.replace_all_in_place(buf, &real_key, &value[0]);
        }
        buf.clone()
    }

    fn replace_all_in_place(&mut self, subject: &mut String, search: &str, replace: &str) {
        let mut pos = 0;
        while let Some(index) = subject[pos..].find(search) {
            let index = index + pos;
            subject.replace_range(index..index + search.len(), replace);
            pos = index + replace.len();
        }
        if pos == 0 {
            self.missing
                .insert(search.to_string(), vec![replace.to_string()]);
        }
    }
}

fn do_proxy(bcc: &mut elvwasm::BitcodeContext) -> CallResult {
    let http_p = &bcc.request.params.http;
    let qp = &http_p.query;
    bcc.log_debug(&format!(
        "In DoProxy hash={} headers={:#?} query params={qp:#?}",
        &bcc.request.q_info.hash, &http_p.headers
    ))?;
    let res = bcc.sqmd_get_json("/request_parameters")?;
    let mut meta_str: String = match String::from_utf8(res) {
        Ok(m) => m,
        Err(e) => {
            return bcc.make_error_with_kind(ErrorKinds::Invalid(format!(
                "failed to parse request params err = {e}"
            )))
        }
    };
    let mut replacer = Replacer {
        missing: qp.clone(),
    };
    let rep = replacer.replace_all(&mut meta_str, qp).as_str().to_owned(); // Change the type of `rep` to `String`
    let replaced: serde_json::Value = serde_json::from_str(&rep)?; // Pass `rep` to `serde_json::from_str`

    let proxy_resp = bcc.proxy_http(Some(json!({ "request": replaced })))?;
    let proxy_resp_json: serde_json::Value =
        serde_json::from_str(std::str::from_utf8(&proxy_resp).unwrap_or("{}"))?;
    let client_response = serde_json::to_vec(&proxy_resp_json["result"])?;
    bcc.callback(200, "application/json", client_response.len())?;
    bcc.write_stream("fos", &client_response)?;
    bcc.make_success_json(&json!(
    {
        "headers" : "application/json",
        "body" : "SUCCESS",
        "result" : 0,
    }))
}

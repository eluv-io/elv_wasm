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
    bcc.make_success_json(&json!({}))
}

mod tests {
    macro_rules! output_raw_pointers {
        ($raw_ptr:ident, $raw_len:ident) => {
            unsafe {
                std::str::from_utf8(std::slice::from_raw_parts($raw_ptr, $raw_len))
                    .unwrap_or("unable to convert")
            }
        };
    }

    #[no_mangle]
    pub extern "C" fn __console_log(ptr: *const u8, len: usize) {
        let out_str = output_raw_pointers!(ptr, len);
        println!("console output : {}", out_str);
    }
    #[no_mangle]
    pub extern "C" fn __host_call(
        bd_ptr: *const u8,
        bd_len: usize,
        ns_ptr: *const u8,
        ns_len: usize,
        op_ptr: *const u8,
        op_len: usize,
        ptr: *const u8,
        len: usize,
    ) -> usize {
        let out_bd = output_raw_pointers!(bd_ptr, bd_len);
        let out_ns = output_raw_pointers!(ns_ptr, ns_len);
        let out_op = output_raw_pointers!(op_ptr, op_len);
        let out_ptr = output_raw_pointers!(ptr, len);
        println!(
            "host call bd = {} ns = {} op = {}, ptr={}",
            out_bd, out_ns, out_op, out_ptr
        );
        1
    }
    #[no_mangle]
    pub extern "C" fn __host_response(ptr: *mut u8) {
        println!("host __host_response ptr = {:?}", ptr);
        let s = r#"{ "result" : {
			"url": "https://www.googleapis.com/customsearch/v1?key=${API_KEY}&q=${QUERY}&cx=${CONTEXT}",
		    "method": "GET",
		    "headers": {
			 "Accept": "application/json",
			 "Content-Type": "application/json"
		   }}}"#;
        unsafe {
            std::ptr::copy(s.as_ptr(), ptr, s.len() + 1);
        }
    }

    #[no_mangle]
    pub extern "C" fn __host_response_len() -> usize {
        println!("host __host_response_len");
        let s = r#"{ "result" : {
			"url": "https://www.googleapis.com/customsearch/v1?key=${API_KEY}&q=${QUERY}&cx=${CONTEXT}",
		    "method": "GET",
		    "headers": {
			 "Accept": "application/json",
			 "Content-Type": "application/json"
		   }}}"#;
        s.len()
    }

    #[no_mangle]
    pub extern "C" fn __host_error_len() -> usize {
        println!("host __host_error_len");
        0
    }

    #[no_mangle]
    pub extern "C" fn __host_error(ptr: *const u8) {
        println!("host __host_error ptr = {:?}", ptr);
    }

    #[no_mangle]
    pub extern "C" fn __guest_response(ptr: *const u8, len: usize) {
        let out_resp = output_raw_pointers!(ptr, len);
        println!("host  __guest_response ptr = {}", out_resp);
    }

    #[no_mangle]
    pub extern "C" fn __guest_error(ptr: *const u8, len: usize) {
        let out_error = output_raw_pointers!(ptr, len);
        println!("host  __guest_error ptr = {}", out_error);
    }

    #[no_mangle]
    pub extern "C" fn __guest_request(op_ptr: *const u8, ptr: *const u8) {
        println!("host __guest_request op_ptr = {:?} ptr = {:?}", op_ptr, ptr);
    }

    #[test]
    fn test_do_proxy() {
        use crate::do_proxy;
        use elvwasm::HttpParams;
        use elvwasm::Request;
        use std::collections::HashMap;

        let mut bcc = elvwasm::BitcodeContext::new(Request::default());
        let mut http_p = HttpParams::default();
        let mut qp = HashMap::new();
        qp.insert("QUERY".to_string(), vec!["fabric".to_string()]);
        qp.insert(
            "API_KEY".to_string(),
            vec!["AIzaSyCppaD53DdPEetzJugaHc2wW57hG0Y5YWE".to_string()],
        );
        qp.insert(
            "CONTEXT".to_string(),
            vec!["012842113009817296384:qjezbmwk0cx".to_string()],
        );
        http_p.query = qp;
        bcc.request.params.http = http_p;
        let res = do_proxy(&mut bcc);
        assert_eq!(res.is_ok(), true);
    }
}

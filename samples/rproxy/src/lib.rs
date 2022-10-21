extern crate elvwasm;
extern crate serde_json;
use serde_json::json;

use elvwasm::{implement_bitcode_module, jpc, register_handler, BitcodeContext, ErrorKinds};

implement_bitcode_module!("proxy", do_proxy);

fn do_proxy<>(bcc: &mut elvwasm::BitcodeContext) -> CallResult {
  let http_p = &bcc.request.params.http;
  let qp = &http_p.query;
  BitcodeContext::log(&format!("In DoProxy hash={} headers={:#?} query params={:#?}",&bcc.request.q_info.hash, &http_p.headers, qp));
  let res = bcc.sqmd_get_json("/request_parameters")?;
  let mut meta_str: String = match String::from_utf8(res){
    Ok(m) => m,
    Err(_e) => {return bcc.make_error_with_kind(ErrorKinds::Invalid("failed to parse request params"));}
  };
  meta_str = meta_str.replace("${API_KEY}", &qp["API_KEY"][0].to_string()).
    replace("${QUERY}", &qp["QUERY"][0].to_string()).
    replace("${CONTEXT}", &qp["CONTEXT"][0].to_string());
  BitcodeContext::log(&format!("MetaData = {}", &meta_str));
  let req:serde_json::Map<String,serde_json::Value> = match serde_json::from_str::<serde_json::Map<String,serde_json::Value>>(&meta_str){
    Ok(m) => m,
    Err(_e) => return bcc.make_error_with_kind(ErrorKinds::Invalid("serde_json::from_str failed")),
  };
  let proxy_resp =  bcc.proxy_http(Some(json!({"request": req})))?;
  let proxy_resp_json:serde_json::Value = serde_json::from_str(std::str::from_utf8(&proxy_resp).unwrap_or("{}"))?;
  let client_response = serde_json::to_vec(&proxy_resp_json["result"])?;
  let id = &bcc.request.id;
  bcc.callback(200, "application/json", client_response.len())?;
  BitcodeContext::write_stream_auto(id.clone(), "fos", &client_response)?;
  bcc.make_success_json(&json!(
    {
        "headers" : "application/json",
        "body" : "SUCCESS",
        "result" : 0,
    }), id)
}

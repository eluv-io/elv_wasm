extern crate elvwasm;
extern crate serde_json;
use serde_json::json;

use elvwasm::{implement_bitcode_module, jpc, make_json_error, register_handler, BitcodeContext, ElvError, ErrorKinds};

implement_bitcode_module!("proxy", do_proxy);

static SQMD_REQUEST: &'static str = "/request_parameters";
static STANDARD_ERROR:&'static str = "no error, failed to acquire error context";

fn do_proxy<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> CallResult {
  let http_p = &bcc.request.params.http;
  let qp = &http_p.query;
  BitcodeContext::log(&format!("In DoProxy hash={} headers={:#?} query params={:#?}",&bcc.request.q_info.hash, &http_p.headers, qp));
  let res = bcc.sqmd_get_json(SQMD_REQUEST)?;
  let mut meta_str: String = match String::from_utf8(res){
    Ok(m) => m,
    Err(e) => {return bcc.make_error(&String::from_utf8(e.as_bytes().to_vec()).unwrap_or(STANDARD_ERROR.to_string()));}
  };
  meta_str = meta_str.replace("${API_KEY}", &qp["API_KEY"][0].to_string()).
    replace("${QUERY}", &qp["QUERY"][0].to_string()).
    replace("${CONTEXT}", &qp["CONTEXT"][0].to_string());
  BitcodeContext::log(&format!("MetaData = {}", &meta_str));
  let req:serde_json::Map<String,serde_json::Value> = match serde_json::from_str::<serde_json::Map<String,serde_json::Value>>(&meta_str){
    Ok(m) => m,
    Err(e) => return make_json_error(ElvError::new_json("serde_json::from_str failed", ErrorKinds::Invalid, e))
  };
  let proxy_resp =  bcc.proxy_http(json!({"request": req}))?;
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

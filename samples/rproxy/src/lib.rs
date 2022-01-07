extern crate elvwasm;
extern crate wapc_guest as guest;
extern crate serde_json;
use serde_json::json;

use guest::{console_log, register_function, CallResult};
use elvwasm::*;

implement_bitcode_module!("proxy", do_proxy);

static SQMD_REQUEST: &'static str = "/request_parameters";
static STANDARD_ERROR:&'static str = "no error, failed to acquire error context";

fn do_proxy<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> CallResult {

  let http_p = &bcc.request.params.http;
  let qp = http_p.query.clone();
  console_log(&format!("In DoProxy hash={} headers={:#?} query params={:#?}",&bcc.request.q_info.hash, &http_p.headers, qp));
  let res = bcc.sqmd_get_json(SQMD_REQUEST)?;
  let mut meta_str: String = match String::from_utf8(res.clone()){
    Ok(m) => m,
    Err(e) => {return bcc.make_error(&String::from_utf8(e.as_bytes().to_vec()).unwrap_or(STANDARD_ERROR.to_string()));}
  };
  meta_str = meta_str.replace("${API_KEY}", &qp["API_KEY"][0].to_string()).
    replace("${QUERY}", &qp["QUERY"][0].to_string()).
    replace("${CONTEXT}", &qp["CONTEXT"][0].to_string());
  console_log(&format!("MetaData = {}", &meta_str));
  let req:serde_json::Map<String,serde_json::Value> = match serde_json::from_str::<serde_json::Map<String,serde_json::Value>>(&meta_str){
    Ok(m) => m,
    Err(e) => return make_json_error(ElvError::new_json("test", ErrorKinds::Invalid, e))
  };
  let proxy_http = json!({"request": req});
  let proxy_resp =  bcc.proxy_http(proxy_http)?;
  let proxy_resp_json:serde_json::Value = serde_json::from_str(std::str::from_utf8(&proxy_resp).unwrap()).unwrap();
  let body_only = &proxy_resp_json["result"];
  let client_response = serde_json::to_vec(&body_only).unwrap();
  let id = bcc.request.id.clone();
  bcc.callback(200, "application/json", client_response.len())?;
  BitcodeContext::write_stream_auto(id.clone(), "fos".to_owned(), &client_response)?;
  let jret = json!(
    {
        "headers" : "application/json",
        "body" : "SUCCESS",
        "result" : 0,
    });
  bcc.make_success_json(&jret, &bcc.request.id)
}

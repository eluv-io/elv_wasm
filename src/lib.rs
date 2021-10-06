
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
use serde::{Deserialize, Serialize};
use serde_json::json;
extern crate wapc_guest as guest;

use std::error::Error;
use std::fmt;

use std::str;

use guest::prelude::*;
use std::collections::HashMap;

use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
  static ref CALLMAP: Mutex<HashMap<String, HandlerFunction>> = Mutex::new(HashMap::new());
}

/*
{
  "id": "b31b79ed-a0fd-4267-9344-e24a3fb463d4",
  "jpc": "1.0",
  "method": "content",
  "params": {
    "http": {
      "headers": {
        "Content-type": ["application/json"]
      },
      "path": "/proxy",
      "query": {
        "API_KEY": "AIzaSyCppaD53DdPEetzJugaHc2wW57hG0Y5YWE",
        "CONTEXT": "012842113009817296384:qjezbmwk0cx",
        "QUERY": "fabric"
      },
      "verb": "unused"
    }
  },
  "QInfo": {
    "hash": "hq__3B4dcp2T2gqYLYPukxMYWsVkQBH5DuA8WxjGd7dcXodzZf15F3VFNd96Zc1B9QszZmKAH",
    "id": "iq__AsQaUWfFcSJFYNyeyVMi6V",
    "qlib_id": "ilibBKAMfarVtkNhJiW8Xou2VW",
    "type": "",
    "write_token": "tqw__K2gHQyYdeMXKq2hTfZmNwPaiaXoEmJYiY6NeyvenGK8pJ6htkG2ggbwe8GjKUu3fC1ZANT3v"
  }
}
*/

#[derive(Debug)]
struct ElvError {
    details: String
}

impl ElvError {
    fn new(msg: &str) -> ElvError {
      ElvError{details: msg.to_string()}
    }
}

impl fmt::Display for ElvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for ElvError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[derive(Serialize, Deserialize,  Clone, Debug)]
pub struct JpcParams {
  pub http: HttpParams
}

#[derive(Serialize, Deserialize,  Clone, Debug)]
pub struct HttpParams {
  pub headers: HashMap<String, Vec<String>>,
  pub path: String,
  pub query: HashMap<String, String>,
  pub verb: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QInfo {
  pub hash: String,
  pub id: String,
  pub qlib_id: String,
  #[serde(rename = "type")]
  pub qtype: String,
  pub write_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Request {
  pub id: String,
  pub jpc: String,
  pub method: String,
  pub params: JpcParams,
  #[serde(rename = "qinfo")]
  pub q_info: QInfo,
}


#[derive(Serialize, Deserialize)]
pub struct Response {
  pub jpc: String,
  #[serde(rename = "params")]
  pub params : serde_json::Value,
  pub id:String,
  pub module:String,
  pub method:String,
}

#[derive(Debug, Clone)]
pub struct BitcodeContext<'a> {
  pub request: &'a Request,
  pub open_streams: Vec<String>,
  pub return_buffer: Vec<u8>,
}

//type MethodHandlerType = fn(bcc: & mut BitcodeContext) -> CallResult;
type HandlerFunction = fn(bcc: & mut BitcodeContext) -> CallResult;
// impl<'b> BitcodeContext<'b> {
//   fn new_stream_internal(self) -> String{
//     match self.call_function("NewStream", serde_json::from_str(&"{}").unwrap(), "ctx"){
//       Ok(f) => {
//         let st: HashMap<&str, serde_json::Value> = serde_json::from_slice(&f).unwrap_or_default();
//         return st[&"stream_id"].to_string();
//       }
//       Err(e) => {
//         return e.to_string();
//       }
//     }

//   }
// }

impl<'a> BitcodeContext<'a> {
  fn new(request: &'a Request) -> BitcodeContext<'a> {
    BitcodeContext {
      request: request,
      open_streams: vec![],
      return_buffer : vec![],
    }
  }

  pub fn write_stream(id:String, stream:String,  src:&'a [u8], len: usize) -> CallResult {
    let mut actual_len = src.len();
    if len != usize::MAX {
      actual_len = len
    }
    let v = serde_json::json!(src[..actual_len]);
    return guest::host_call(&id, &stream, &"Write".to_string(), &serde_json::to_vec(&v).unwrap());
  }

  pub fn write_stream_auto(id:String, stream:String,  src:&'a [u8]) -> CallResult {
    return guest::host_call(&id, &stream, &"Write".to_string(), src);
  }

  pub fn callback(&'a self, status:usize, content_type:&str, size:usize) -> CallResult{
    let v = json!(
      {"http" : {
        "status": status,
        "headers": {
          "Content-Type": content_type,
          "Content-Length": size,
        }
        }
      }
    );
    let method  = "Callback";
    return self.call_function(method, v, "ctx");
  }

  pub fn read_stream(&'a mut self, stream_to_read:String, sz:usize) -> CallResult {
        let input = serde_json::json![sz];
        return guest::host_call(self.request.id.as_str(),stream_to_read.as_str(), &"Read", &serde_json::to_vec(&input).unwrap());
  }

  pub fn temp_dir(&'a mut self) -> String {
    return str::from_utf8(&self.call("TempDir", &"{}", &"ctx".as_bytes()).unwrap()).unwrap().to_string();
  }

  pub fn close_stream(&'a mut self, sid : String) -> CallResult{
    return self.call_function(&"CloseStream", serde_json::Value::String(sid), &"ctx");
  }

  fn add_stream_to_close(&'a mut self, s:&str){
    self.open_streams.push(s.to_string());

  }

  pub fn make_success(&'a self, msg:&str, idStr:&str) -> CallResult {
    let js_ret = json!({"jpc":"1.0", "id": idStr, "result" : msg});
    let v = serde_json::to_vec(&js_ret);
    return Ok(v.unwrap());
  }

  pub fn make_success_bytes(&'a self, msg:&[u8], idStr:&str) -> CallResult {
    let res:serde_json::Value = serde_json::from_slice(msg).unwrap();
    let js_ret = json!({"jpc":"1.0", "id": idStr, "result" : res});
    let v = serde_json::to_vec(&js_ret);
    return Ok(v.unwrap());
  }

  pub fn sqmd_get_json(&'a self, s:&'a str) -> CallResult {
    //let spath = format!("{\"{}\" : \"{}\"}", "path", s);
    let sqmd_get = json!({"path": s});
    let method = "SQMDGet";
    return self.call_function(&method, sqmd_get, &"core");
  }

  pub fn proxy_http(&'a self, v:serde_json::Value) -> CallResult {
    let method = "ProxyHttp";
    let proxy_result = self.call_function(&method, v, &"ext").unwrap();
    let id = self.request.id.clone();
    return self.make_success_bytes(&proxy_result, &id);
  }

  // pub fn new_stream(&'a mut self) -> String{
  //   let sid = self.clone().new_stream_internal();
  //   self.add_stream_to_close(&sid);
  //   return sid.to_string();
  // }

  fn call(&'a mut self, ns: &str, op: &str, msg: &[u8]) -> CallResult{
    return guest::host_call(self.request.id.as_str(),ns,op,msg);
  }

  pub fn call_function(&'a self, fn_name:&str , params:serde_json::Value, module:&str) -> CallResult {
    //`{ "jpc" : "1.0", "params" : __PARAMS__, "id" : __ID__, "module" : __MODULE__, "method" : __METHOD__}`;
    let response = &Response{
      jpc:"1.0".to_string(),
      id:self.request.id.clone(),
      module:module.to_string(),
      method : fn_name.to_string(),
      params:params,
    };
    let call_val = serde_json::to_vec(response).unwrap();
    let call_str = str::from_utf8(&call_val).unwrap();
    let call_str2 = serde_json::to_string(response).unwrap();

    console_log("CALL STRING");
    console_log(call_str);
    console_log(&call_str2);
    return guest::host_call(self.request.id.as_str(),module, fn_name, &call_val);
  }
}


#[no_mangle]
pub fn register_handler(name: &str, h: HandlerFunction) {
  CALLMAP.lock().unwrap().insert(name.to_string(), h);
}

#[no_mangle]
pub fn jpc<'a>(_msg: &'a [u8]) -> CallResult {
  guest::console_log(&"Hello");
  let input_string = str::from_utf8(_msg)?;
  guest::console_log(&"Hello Again");
  guest::console_log(&input_string);

  let json_params: Request = serde_json::from_str(input_string)?;
  guest::console_log(&"Hello Again");

  let mut bcc = BitcodeContext::new(&json_params);
  let j = serde_json::to_string(&json_params)?;
  guest::console_log(&"Hello Again");
  guest::console_log(&j);
  let split_path: Vec<&str> = bcc.request.params.http.path.as_str().split('/').collect();
  match CALLMAP.lock().unwrap().get(split_path[1]) {
    Some(f) => {
      return f(& mut bcc);
    }
    None => {return Err(Box::new(ElvError::new("No valid path provided")));}
  };
}



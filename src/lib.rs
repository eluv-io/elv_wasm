
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
use serde::{Deserialize, Serialize};
extern crate wapc_guest as guest;

use std::str;

use guest::prelude::*;
use std::collections::HashMap;

use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
  static ref CALLMAP: Mutex<HashMap<String, MethodHandlerType>> = Mutex::new(HashMap::new());
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

#[derive(Serialize, Deserialize)]
pub struct JpcParams {
  http: HttpParams
}

#[derive(Serialize, Deserialize)]
pub struct HttpParams {
  pub headers: HashMap<String, Vec<String>>,
  pub path: String,
  pub query: HashMap<String, String>,
  pub verb: String,
}

#[derive(Serialize, Deserialize)]
pub struct QInfo {
  pub hash: String,
  pub id: String,
  pub qlib_id: String,
  #[serde(rename = "type")]
  pub qtype: String,
  pub write_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct Request {
  pub id: String,
  pub jpc: String,
  pub method: String,
  pub params: JpcParams,
  pub QInfo: QInfo,
}

pub struct BitcodeContext<'a> {
  pub request: &'a Request,
  pub openStreams: Vec<String>,
  pub hc: HostCallType,
}

type HostCallType = fn(binding: &str, ns: &str, op: &str, msg: &[u8]) -> CallResult;
type MethodHandlerType = fn(bcc: &BitcodeContext) -> CallResult;

impl<'a> BitcodeContext<'a> {
  fn new(request: &'a Request, hc: HostCallType) -> BitcodeContext<'a> {
    BitcodeContext {
      request: request,
      hc: hc,
      openStreams: vec![],
    }
  }

  fn CallFunction(&self) -> &'a [u8] {
    let path: &String = &self.request.params.http.path;
    let method: &String = &path.as_str().split("/").collect();
    static a: Vec<u8> = vec![];
    &a[..]
  }
}

type HandlerFunction = fn(&BitcodeContext) -> CallResult;

fn register_handler(name: &str, h: HandlerFunction) {
  CALLMAP.lock().unwrap().insert(name.to_string(), h);
}

fn jpc<'a>(_msg: &'a [u8]) -> CallResult {
  guest::console_log(&"Hello");
  let input_string = str::from_utf8(_msg)?;
  guest::console_log(&"Hello Again");
  let json_params: Request = serde_json::from_str(input_string)?;
  guest::console_log(&"Hello Again");

  let bcc = BitcodeContext::new(&json_params, guest::host_call);
  let j = serde_json::to_string(&json_params)?;
  guest::console_log(&"Hello Again");
  guest::console_log(&j);
  let split_path: Vec<&str> = bcc.request.params.http.path.as_str().split('/').collect();
  match CALLMAP.lock().unwrap().get(split_path[1]) {
    Some(f) => f(&bcc),
    None => std::result::Result::Ok(vec![])
  };

  //json_params.as_object()?
  //  let _res = host_call("myBinding", "sample:Host", "Call", b"hello")?;
  Ok(vec![])
}



//! elvwasm contains and collects the bitcode extension API for the Eluvio content fabric. </br>
//! The library is intended to be built as wasm and the resultant part uploaded to the content fabric.
//! The main entry point for each client module is implemented by [jpc] which automatically creates and dispatches
//! requests to the [BitcodeContext] </br>
//! Example
/*!
  ```rust
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
      let id = bcc.request.id.clone();
      bcc.callback(200, "application/json", proxy_resp.len())?;
      BitcodeContext::write_stream_auto(id.clone(), "fos".to_owned(), &proxy_resp)?;
      return bcc.make_success("SUCCESS");
  }
  ```

  To Build binaries </br>
    *cargo build --all --features "host-wasm"* </br>
  To Build samples </br>
    *cd samples* </br>
    *cargo build --target wasm32-unknown-unknown* </br>
  </br>
  test </br>
    *target/debug/mock ./samples/target/wasm32-unknown-unknown/debug/deps/rproxy.wasm ./samples/fabric.json* 
*/

extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate wapc_guest as guest;
#[macro_use(defer)] extern crate scopeguard;

mod bccontext;
pub use self::bccontext::*;

//use guest::console_log;

use std::str;

use guest::prelude::*;
use std::collections::HashMap;

use lazy_static::lazy_static;
use std::sync::Mutex;


lazy_static! {
  static ref CALLMAP: Mutex<HashMap<String, HandlerFunction>> = Mutex::new(HashMap::new());
}

#[doc(hidden)]
#[macro_export]
macro_rules! output_raw_pointers {
  ($raw_ptr:ident, $raw_len:ident) => {
        unsafe { std::str::from_utf8(std::slice::from_raw_parts($raw_ptr, $raw_len)).unwrap_or("unable to convert")}
  }
}

/// This macro creates the necessary exports for WAPC to satisfy the linker
/// An example of its usage is in ./mock/src/main.rs but can be used in a custom mock as well
#[macro_export]
macro_rules! implement_mock_fabric {
  () => {
    #[no_mangle]
    pub extern "C" fn __console_log(ptr: *const u8, len: usize){
      let out_str = output_raw_pointers!(ptr,len);
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
        println!("host call bd = {} ns = {} op = {}, ptr={}", out_bd, out_ns, out_op, out_ptr);
        0
    }

    #[no_mangle]
    pub extern "C" fn __host_response(ptr: *const u8){
      println!("host __host_response ptr = {:?}", ptr);
    }

    #[no_mangle]
    pub extern "C" fn __host_response_len() -> usize{
      println!("host __host_response_len");
      0
    }

    #[no_mangle]
    pub extern "C" fn __host_error_len() -> usize{
      println!("host __host_error_len");
      0
    }

    #[no_mangle]
    pub extern "C" fn __host_error(ptr: *const u8){
      println!("host __host_error ptr = {:?}", ptr);
    }

    #[no_mangle]
    pub extern "C" fn __guest_response(ptr: *const u8, len: usize){
      let out_resp = output_raw_pointers!(ptr,len);
      println!("host  __guest_response ptr = {}", out_resp);
    }

    #[no_mangle]
    pub extern "C" fn __guest_error(ptr: *const u8, len: usize){
      let out_error = output_raw_pointers!(ptr,len);
      println!("host  __guest_error ptr = {}", out_error);
    }

    #[no_mangle]
    pub extern "C" fn __guest_request(op_ptr: *const u8, ptr: *const u8){
      println!("host __guest_request op_ptr = {:?} ptr = {:?}", op_ptr, ptr);

    }
  };
}

#[macro_export]
macro_rules! implement_bitcode_module {
  ($handler_name:literal, $handler_func:ident) => {
    #[no_mangle]
    pub extern "C" fn wapc_init() {
      register_handler($handler_name, $handler_func);
      register_function("_jpc", jpc);
    }
  }
}

// The following are mearly intended to verify internal consistency.  There are no actual calls made
// but the tests verify that the json parsing of the http message is correct
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use serde_json::*;
    pub use self::bccontext::*;
    pub use self::bccontext::{QList};
    use wapc::*;
    use wapc_guest::*;
    implement_mock_fabric!();

    fn handler_for_test(bcc: & mut BitcodeContext) -> CallResult{
      bcc.make_success("DONE")
    }


    #[test]
    fn test_basic_http(){
        register_handler("test_handler", handler_for_test);
        let test_json = json!({
          "id" : "dummydummy",
          "jpc" : "1.0",
          "method" : "GET",
          "params" : {
            "http" : {
              "path" : "/testing",
            },
          },
        });
        match serde_json::to_vec(&test_json){
          Ok(x) => {
            let res = jpc(&x);
            match res{
              Ok(_) => {
              },
              Err(err) => {
                panic!("failed test_http err = {:?}", err);
              }
            }
          },
          Err(err) =>{
            panic!("failed test_http err = {:?}", err);
          }
        };
    }

    #[test]
    fn test_basic_http_failure(){
      register_handler("test_handler", handler_for_test);
      // path missing
      let test_json = json!({
        "id" : "dummydummy",
        "jpc" : "1.0",
        "method" : "GET",
        "params" : {
          "http" : {
          },
        },
      });
      match serde_json::to_vec(&test_json){
        Ok(x) => {
          let res = jpc(&x);
          match res{
            Ok(k) => {
                let mut res_json:serde_json::Map<String, serde_json::Value> = serde_json::from_slice(&k).unwrap();
                let mut err_json:serde_json::Map<String, serde_json::Value> = serde_json::from_value(res_json["error"].take()).unwrap();
                assert_eq!(err_json["op"], "BadHttpParams");
                let err_json_data:serde_json::Map<String, serde_json::Value> = serde_json::from_value(err_json["data"].take()).unwrap();
                assert_eq!(err_json_data["op"], "BadHttpParams");
            },
            Err(err) => {
              panic!("failed test_http err = {:?}", err);
            }
          }
        },
        Err(err) =>{
          panic!("failed test_http err = {:?}", err);
        }
      };
  }
}





type HandlerFunction = fn(bcc: & mut BitcodeContext) -> CallResult;


/// register_handler adjusts the global static call map to associate a bitcode module with a path
/// this map is used by jpc to implement bitcode calls
#[no_mangle]
pub fn register_handler(name: &str, h: HandlerFunction) {
  CALLMAP.lock().unwrap().insert(name.to_string(), h);
}

#[cfg(not(test))]
fn elv_console_log(s:&str){
  console_log(s)
}

#[cfg(test)]
fn elv_console_log(s:&str){
  println!("{}", s)
}

/// jpc is the main entry point into a wasm bitcode for the web assembly procedure calls
/// this function will
/// # Steps
///   * parse the input for the appropriately formatted json
///   * construct a BitcodeContext from the json
///   * attempt to call the method using the incomming path
///   * return results to the caller
#[no_mangle]
pub fn jpc(_msg: &[u8]) -> CallResult {
  elv_console_log("In jpc");
  let input_string = str::from_utf8(_msg)?;
  elv_console_log(&format!("parameters = {}", input_string));
  let json_params: Request = match serde_json::from_str(input_string){
    Ok(m) => {m},
    Err(err) => {
      return make_json_error(ElvError::new_json("parse failed for http" , ErrorKinds::BadHttpParams, err));
    }
  };
  elv_console_log("Request parsed");
  let mut bcc = BitcodeContext::new(&json_params);
  elv_console_log("Parameters parsed");
  let split_path: Vec<&str> = bcc.request.params.http.path.as_str().split('/').collect();
  elv_console_log(&format!("splitpath={:?}", split_path));
  let cm = CALLMAP.lock();
  match cm.unwrap().get(split_path[1]) {
    Some(f) => {
      match f(& mut bcc){
        Ok(m) => {
          elv_console_log(&format!("here and m={:?}", m));
          Ok(m)
        },
        Err(err) => {
          bcc.make_error_with_error("parse failed for http" , ErrorKinds::Invalid, &*err)
        }
      }
    }
    None => {
      elv_console_log(&format!("Failed to find path {}", split_path[1]));
      bcc.make_error_with_kind("No valid path provided", ErrorKinds::BadHttpParams)
    }
  }
}



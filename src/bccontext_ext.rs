extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate thiserror;
extern crate wapc_guest as guest;

use crate::{BitcodeContext};

use serde_json::{json, Value};
use std::str;
use guest::CallResult;

#[macro_export]
macro_rules! implement_ext_func {
    (
      $(#[$meta:meta])*
      $handler_name:ident,
      $fabric_name:literal,
      $module_name:literal
    ) => {
      $(#[$meta])*
      pub fn $handler_name(&'a self, v:Option<serde_json::Value>) -> CallResult {
        let v_call = match v{
          Some(v) => v,
          None => Value::default(),
        };
        let method = $fabric_name;
        let impl_result = self.call_function(method, v_call, $module_name)?;
        let id = self.request.id.clone();
        self.make_success_bytes(&impl_result, &id)
      }
    }
  }

impl<'a> BitcodeContext{
    implement_ext_func!(
        /// proxy_http proxies an http request in case of CORS issues
        /// # Arguments
        /// * `v` : a JSON Value
        ///
        /// ```
        /// use serde_json::json;
        ///
        /// fn do_something<'s>(bcc: &'s elvwasm::BitcodeContext) -> wapc_guest::CallResult {
        ///   let v = json!({
        ///         "request_parameters" : {
        ///         "url": "https://www.googleapis.com/customsearch/v1?key=AIzaSyCppaD53DdPEetzJugaHc2wW57hG0Y5YWE&q=fabric&cx=012842113009817296384:qjezbmwk0cx",
        ///         "method": "GET",
        ///         "headers": {
        ///           "Accept": "application/json",
        ///           "Content-Type": "application/json"
        ///         }
        ///      }
        ///   });
        ///   bcc.proxy_http(Some(v))
        /// }
        /// ```
        /// # Returns
        /// * slice of [u8]
        proxy_http,
        "ProxyHttp",
        "ext"
    );

    pub fn start_bitcode_lro(&'a self, function: &str, args: &serde_json::Value) -> CallResult {
        let params = json!({ "function": function,  "args" : args});
        self.call_function("StartBitcodeLRO", params, "ext")
    }

}

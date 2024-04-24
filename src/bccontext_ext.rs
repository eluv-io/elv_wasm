extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate thiserror;
extern crate wapc_guest as guest;

use crate::BitcodeContext;

use guest::CallResult;
use serde_json::{json, Value};
use std::str;

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

impl<'a> BitcodeContext {
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
        ///
        /// # Returns
        /// * slice of [u8]
        ///
        /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/rproxy/src/lib.rs#L26)
        ///
        proxy_http,
        "ProxyHttp",
        "ext"
    );

    implement_ext_func!(
        /// rest_call calls a predefined rest server that has to be configured in the fabric for a specific node
        /// # Arguments
        /// * `v` : a JSON Value
        ///
        /// ```
        /// use serde_json::json;
        ///
        /// fn do_something<'s>(bcc: &'s elvwasm::BitcodeContext) -> wapc_guest::CallResult {
        ///   let v = json!({
        ///         "request_parameters" : {
        ///         "url": "node_service_1/rep/tagger",
        ///         "method": "GET",
        ///         "headers": {
        ///           "Accept": "application/json",
        ///           "Content-Type": "application/json"
        ///         }
        ///      }
        ///   });
        ///   bcc.rest_call(Some(v))
        /// }
        /// ```
        ///
        /// # Returns
        /// * slice of [u8]
        ///
        /// [Example](FIXME)
        ///
        rest_call,
        "RestCall",
        "ext"
    );

    /// start_bitcode_lro initiates a long running operation on the fabric.  Currently the lro implementation
    /// constrains the callback to be in the same bitcode module as the initiator.
    /// # Arguments
    /// * `module` : &str the bitcode module on which to call the function. Empty for "current module".
    /// * `function` : &str the function to call in the given bitcode module.
    /// * `args` : JSON value containing the arguments to pass the callback
    ///
    /// # Returns
    /// * slice of [u8]
    ///
    ///  [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/lro/src/lib.rs#L16)
    ///
    pub fn start_bitcode_lro(&'a self, module: &str, function: &str, args: &serde_json::Value) -> CallResult {
        let params = json!({ "module": module, "function": function,  "args" : args});
        self.call_function("StartBitcodeLRO", params, "lro")
    }
}

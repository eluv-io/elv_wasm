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


    /// ffmpeg_run - runs ffmpeg server side
    /// # Arguments
    /// * `cmdline` - a string array with ffmpeg command line arguments
    /// - note the ffmpeg command line may reference files opened using new_file_stream.
    /// eg
    /// ```
    ///  fn ffmpeg_run_watermark(bcc:&elvwasm::BitcodeContext, height:&str, input_file:&str, new_file:&str, watermark_file:&str, overlay_x:&str, overlay_y:&str) -> wapc_guest::CallResult{
    ///     let base_placement = format!("{}:{}",overlay_x,overlay_y);
    ///     let scale_factor = "[0:v]scale=%SCALE%:-1[bg];[bg][1:v]overlay=%OVERLAY%";
    ///     let scale_factor = &scale_factor.replace("%SCALE%", height).to_string().replace("%OVERLAY%", &base_placement).to_string();
    ///     if input_file == "" || watermark_file == "" || new_file == ""{
    ///       let msg = "parameter validation failed, one file is empty or null";
    ///       return bcc.make_error(msg);
    ///     }
    ///     bcc.ffmpeg_run(["-hide_banner","-nostats","-y","-i", input_file,"-i", watermark_file,"-filter_complex", scale_factor,"-f", "singlejpeg", new_file].to_vec())
    ///  }
    /// ```
    pub fn ffmpeg_run(&'a self, cmdline: Vec<&str>) -> CallResult {
        let params = json!({ "stream_params": cmdline });
        self.call_function("FFMPEGRun", params, "ext")
    }

    pub fn start_bitcode_lro(&'a self, function: &str, args: &serde_json::Value) -> CallResult {
        let params = json!({ "function": function,  "args" : args});
        self.call_function("StartBitcodeLRO", params, "ext")
    }

    pub fn call_external_bitcode(&'a self, function: &str, args: &serde_json::Value, object_hash:&str,code_part_hash:&str) -> CallResult {
        let params = json!({
            "jpc" : "1.0",
            "id" : self.request.id,
            "method" : format!("/{function}"),
            "params" : args,
            "qinfo" : self.request.q_info.clone(),
        });
        let params = json!({ "function": function,  "params" : params, "object_hash" : object_hash, "code_part_hash" : code_part_hash});
        self.call_function("CallExternalBitcode", params, "ext")
    }

}
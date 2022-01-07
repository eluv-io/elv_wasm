use serde::{Deserialize, Serialize};
use serde_json::json;
//use guest::console_log;

extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate wapc_guest as guest;


use serde::ser::{Serializer, SerializeStruct};

use std::error::Error;
use std::fmt;
use std::fmt::Debug;

use std::str;
use std::collections::HashMap;


use guest::prelude::*;
use guest::CallResult;


macro_rules! enum_str {
    (enum $name:ident {
        $($variant:ident = $val:expr),*,
    }) => {
        #[derive(Clone, Serialize, Copy)]
        pub enum $name {
            $($variant = $val),*
        }

        impl $name {
            fn name(&self) -> &'static str {
                match self {
                    $($name::$variant => stringify!($variant)),*
                }
            }
        }
    };
  }

  // Errorkinds define the category of content fabric errors that exist.  These errors define categories that can be searched in the
  // content fabric logs (qfab.log)
  enum_str! {
     enum ErrorKinds {
      Other = 0x00,
      NotImplemented = 0x01,
      Invalid = 0x02,
      Permission = 0x03,
      IO = 0x04,
      Exist = 0x05,
      NotExist = 0x06,
      IsDir = 0x07,
      NotDir = 0x08,
      Finalized = 0x09,
      NotFinalized = 0x0a,
      BadHttpParams = 0x0b,
    }
  }

  impl fmt::Display for ErrorKinds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "{}", self.name())
    }
  }

  impl ErrorKinds{
    fn describe(&self) -> &str{
      self.name()
    }
  }

  pub struct NoSubError{
  }

  impl Error for NoSubError {
    fn description(&self) -> &str {
        "No Sub Error"
    }
  }

  impl std::fmt::Display for NoSubError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        return write!(f,"No Sub Error");
    }
  }

  impl std::fmt::Debug for NoSubError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      return write!(f,"No Sub Error");
    }
  }

  /// Defines the structure of an error in WASM bitcode.  The structure mimics the content fabric error structure and these errors
  /// will be returned to the fabric calling code and translated to real content-fabric errors
  #[derive(Clone)]
  pub struct ElvError<T> {
      details: String,
      kind: ErrorKinds,
      description : String,
      json_error : Option<T>,
  }

  impl<T> ElvError<T>{
      pub fn new_json(msg: &str, kind: ErrorKinds, sub_err : T) -> ElvError<T> {
        ElvError{
          details:  msg.to_string(),
          kind,
          description: format!("{{ details : {}, kind : {} }}", msg, kind),
          json_error : Some(sub_err),
        }
      }
      pub fn new(msg: &str, kind: ErrorKinds) -> ElvError<T>{
        ElvError{
          details:  msg.to_string(),
          kind,
          description: format!("{{ details : {}, kind : {} }}", msg, kind),
          json_error: None,
        }
    }
    pub fn new_with_err(msg: &str, kind: ErrorKinds, err:T) -> ElvError<T>{
      ElvError{
        details:  msg.to_string(),
        kind,
        description: format!("{{ details : {}, kind : {} }}", msg, kind),
        json_error: Some(err),
      }
    }
  }

  trait Kind {
    fn kind(&self) -> ErrorKinds;
    fn desc(&self) -> String;
  }

  impl<T:Error> Kind for ElvError<T>{
    fn kind(&self) -> ErrorKinds{
      self.kind
    }
    fn desc(&self) -> String{
      self.description.clone()
    }
  }


  impl<T> std::error::Error for ElvError<T> where
  T: fmt::Display + Debug {
    fn description(&self) -> &str {
        self.kind.describe()
    }
  }

  impl<T> fmt::Display for ElvError<T> where
  T: fmt::Display + Debug {
      fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.json_error{
         Some(e) => {return write!(f,"{{ details : {}, kind : {}, sub_error : {:?} }}", self.details, self.kind, e)},
         None => {return write!(f,"{{ details : {}, kind : {}, sub_error : {{}} }}", self.details, self.kind)}
        };
      }
  }

  impl<T:fmt::Display + Debug> Serialize for ElvError<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where T:fmt::Display + Debug, S:Serializer,
    {
      let mut state = serializer.serialize_struct("ElvError", 3)?;
      state.serialize_field("message", &self.details)?;
      state.serialize_field("op", &self.kind)?;
      match &self.json_error{
        Some(e) => {
          let mut error_details = HashMap::<String,String>::new();
          error_details.insert("op".to_string(), format!("{}", self.kind));
          error_details.insert("rust_error".to_string(), format!("{:?}", &e));
          state.serialize_field("data", &error_details)?;
        },
        None => {
          state.serialize_field("data", "no extra error info")?;
        },
      };
      state.end()
    }
  }

  impl<T>  Debug for ElvError<T>  where
  T: fmt::Display + Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match &self.json_error{
       Some(e) => {return write!(f,"{{ details : {}, kind : {}, sub_error : {:?} }}", self.details, self.kind, e)},
       None => {return write!(f,"{{ details : {}, kind : {}, sub_error : {{}} }}", self.details, self.kind)}
      };
    }
  }

  impl<T:Error> From<T> for ElvError<T> {
    fn from(err: T) -> ElvError<T> {
      ElvError::<T> {
            details: err.to_string(),
            description:"".to_string(),
            json_error:None,
            kind:ErrorKinds::Invalid,
        }
    }
  }


#[cfg(not(test))]
fn elv_console_log(s:&str){
  console_log(s)
}

#[cfg(test)]
fn elv_console_log(s:&str){
  println!("{}", s)
}

/// make_json_error translates the bitcode [ElvError<T>] to an error response to the client
/// # Arguments
/// * `err`- the error to be translated to a response
pub fn make_json_error<T:Error>(err:ElvError<T>) -> CallResult {
    elv_console_log(&format!("error={}", err));
    let msg = json!(
      {"error" :  err }
    );
    let vr = serde_json::to_vec(&msg)?;
    let out = str::from_utf8(&vr)?;
    elv_console_log(&format!("returning a test {}", out));
    Ok(vr)
  }


/// Q is a bitcode representation of an individual piece of content from the fabric
#[derive(Serialize, Deserialize,  Clone, Debug, Default)]
pub struct Q{
  pub id:String,
  pub hash:String,
  #[serde(default)]
  pub write_token:String,
  #[serde(rename = "type")]
  pub q_type:String,
  pub qlib_id:String,
  #[serde(default)]
  pub meta:serde_json::Value,
}

/// Bitcode representation of a fabric size error
#[derive(Serialize, Deserialize,  Clone, Debug)]
pub struct QError{
  pub error:String,
  #[serde(default)]
  pub item:Q,
}

/// QRef is a bitcode representation of versioned content from the fabric
#[derive(Serialize, Deserialize,  Clone, Debug, Default)]
pub struct QRef{
  pub id:String,
  pub versions:Vec<Q>,
}

/// Bitcode representation of a full content listing given an optional filter
#[derive(Serialize, Deserialize,  Clone, Debug)]
pub struct QList{
  #[serde(default)]
  pub filter:String,
  pub contents: Vec<QRef>,
  #[serde(default)]
  pub errors : Vec<QError>
}

/// Bitcode representation of a fabric FileStream
#[derive(Serialize, Deserialize,  Clone, Debug)]
pub struct FileStream {
  /// bitcode stream handle
  pub stream_id:String,
  /// fabric file path
  pub file_name:String,
}

/// Bitcode representation of the size of a fabric stream
#[derive(Serialize, Deserialize,  Clone, Debug)]
pub struct FileStreamSize {
  pub file_size:usize,
}

/// Bitcode representation of the JPC (JSON Procedure Call) parameters from a client request
#[derive(Serialize, Deserialize,  Clone, Debug)]
pub struct JpcParams {
  pub http: HttpParams
}

/// Bitcode representation of the http parameters from a client request
#[derive(Serialize, Deserialize,  Clone, Debug)]
pub struct HttpParams {
  #[serde(default)]
  pub headers: HashMap<String, Vec<String>>,
  pub path: String,
  #[serde(default)]
  pub query: HashMap<String, Vec<String>>,
  pub verb: String,
  #[serde(default)]
  pub fragment: String,
  #[serde(default)]
  pub content_length : usize,
  #[serde(default)]
  pub client_ip : String,
  #[serde(default)]
  pub self_url : String,
  #[serde(default)]
  pub proto : String,
  #[serde(default)]
  pub host : String,
}

/// Bitcode representation of a content sans meta data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QInfo {
  #[serde(default)]
  pub hash: String,
  #[serde(default)]
  pub id: String,
  pub qlib_id: String,
  #[serde(rename = "type")]
  pub qtype: String,
  #[serde(default)]
  pub write_token: String,
}

impl QInfo {
  pub fn qhot(&self) -> String{
    let s:String;
    if self.write_token != "" {
      s = self.write_token.to_string();
    }else{
      s = self.hash.to_string();
    }
    s
  }
}

/// Bitcode representation of a incomming client request
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Request {
  pub id: String,
  pub jpc: String,
  pub method: String,
  pub params: JpcParams,
  #[serde(rename = "qinfo")]
  pub q_info: QInfo,
}

/// Bitcode representation of a request back to the fabric as a consequnce of processing the request
/// In order for bitcode to respond to a primary request from a client, the bitcode must gather info
/// from the fabric for processing.  This structure represents the data to such a call.
#[derive(Serialize, Deserialize)]
pub struct Response {
  pub jpc: String,
  #[serde(rename = "params")]
  pub params : serde_json::Value,
  pub id:String,
  pub module:String,
  pub method:String,
}


/// This structure encapsulates all communication with the Eluvio content fabric.  A new BitcodeContext
/// is automatically created during the processing of the http request.  During initialization, all context
/// data is acquired from the http request.  The BitcodeContext provides 2 way communication to the content fabric.
/// There is convenience impl method [BitcodeContext::call_function] that allows the fabric to be accessed via a known set of APIs.
#[derive(Debug, Clone)]
pub struct BitcodeContext<'a> {
  pub request: &'a Request,
  pub return_buffer: Vec<u8>,
}

impl<'a> BitcodeContext<'a> {
    pub fn new(request: &'a Request) -> BitcodeContext<'a> {
      BitcodeContext {
        request,
        return_buffer : vec![],
      }
    }

    /// write_stream writes a u8 slice of specified length to a fabric stream
    /// # Arguments
    /// * `id`-    a unique identifier (can use BitcodeContext's request id)
    /// * `stream`-  the fabric stream to write to [BitcodeContext::new_stream]
    /// * `src`-  a u8 slice to write
    /// * `len` -  length of the slice to write
    /// # Returns
    /// utf8 bytes stream containing json
    /// { "written" : bytes }
    pub fn write_stream(&'a self, id:&str, stream:&str,  src:&'a [u8], len: usize) -> CallResult {
      let mut actual_len = src.len();
      if len != usize::MAX {
        actual_len = len
      }
      let v = serde_json::json!(src[..actual_len]);
      let jv = &serde_json::to_vec(&v)?;
      host_call(id, stream, &"Write".to_string(), jv)
    }

    /// write_stream writes a u8 slice to a fabric stream
    /// # Arguments
    /// * `id`-    a unique identifier (can use BitcodeContext's request id)
    /// * `stream`-  the fabric stream to write to [BitcodeContext::new_stream]
    /// * `src`-  a u8 slice to write
    /// # Returns
    /// utf8 bytes stream containing json
    /// { "written" : bytes }
    pub fn write_stream_auto(id:String, stream:String,  src:&'a [u8]) -> CallResult {
      host_call(&id, &stream, &"Write".to_string(), src)
    }

    /// write_part_to_stream writes the content of a part to to a fabric stream
    /// # Arguments
    /// * `stream_id`-    stream identifier from new_stream or the like
    /// * `off`-  offset into the file (0 based)
    /// * `len`-  length of part to write
    /// * `qphash` - part hash to write
    /// # Returns
    /// utf8 bytes stream containing json
    /// { "written" : count-of-bytes-written }
    pub fn write_part_to_stream(&'a self, stream_id:String, qphash:String, offset:i64, length:i64) -> CallResult{
      let msg = json!(
        {
          "stream_id" :  stream_id,
          "off":offset,
          "len" : length,
          "qphash":qphash,
       }
      );
      self.call_function("QWritePartToStream", msg, "core")
    }

    /// read_stream reads usize bytes from a fabric stream returning a slice of [u8]
    /// # Arguments
    /// * `stream_to_read`-  the fabric stream to read from
    /// * `sz`-  usize size of bytes
    /// # Returns
    /// utf8 bytes stream containing json
    /// {
    ///   "return" : { "read" : byte-count-read },
    ///   "result" : "base64 encoded string"
    ///  }
    pub fn read_stream(&'a mut self, stream_to_read:String, sz:usize) -> CallResult {
          let input = serde_json::json![sz];
          let input_json = serde_json::to_vec(&input)?;
          host_call(self.request.id.as_str(),stream_to_read.as_str(), "Read", &input_json)
    }

    pub fn temp_dir(&'a mut self) -> CallResult {
      let temp_dir_res = self.call("TempDir", "{}", "ctx".as_bytes())?;
      Ok(temp_dir_res)
    }

    /// callback issues a Callback on the fabric setting up an expectation that the output stream
    /// contains a specified sized buffer
    /// # Arguments
    /// * `status`-    the http status of the call
    /// * `content-type`-     output buffer contents
    /// * `size`-  size of the output contents
    /// # Returns
    /// the checksum as hex-encoded string
    pub fn callback(&'a self, status:usize, content_type:&str, size:usize) -> CallResult{
      let v = json!(
        {"http" : {
          "status": status,
          "headers": {
            "Content-Type": [content_type],
            "Content-Length": [size.to_string()],
          }
          }
        }
      );
      let method  = "Callback";
      self.call_function(method, v, "ctx")
    }


    /// checksum_part calculates a checksum of a given content part.
    /// # Arguments
    /// * `sum_method`-    checksum method ("MD5" or "SHA256")
    /// * `qphash`-        hash of the content part to checksum
    /// # Returns
    /// the checksum as hex-encoded string
    pub fn checksum_part(&'a self, sum_method:&str, qphash:&str) -> CallResult{

      let j = json!(
        {
          "method" : sum_method,
          "qphash" : qphash
        }
      );

      self.call_function("QCheckSumPart", j, "core")
    }


    /// checksum_file calculates a checksum of a file in a file bundle
    /// # Arguments
    /// * `sum_method`-    checksum method ("MD5" or "SHA256")
    /// * `file_path`-     the path of the file in the bundle
    /// # Returns
    /// the checksum as hex-encoded string
    pub fn checksum_file(&'a self, sum_method:&str, file_path:&str) -> CallResult{

      let j = json!(
        {
          "method" : sum_method,
          "file_path" : file_path,
        }
      );

      self.call_function("QCheckSumFile", j, "core")
    }

    /// q_list_content_for calculates a content fabric QList for a given libid
    /// # Arguments
    /// * `qlibid`-    libid to be listed
    /// # Returns
    /// [Vec<u8>] parseable to [QList]
    /// e.g.
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   let res = bcc.q_list_content_for(&bcc.request.q_info.qlib_id)?;
    ///   let qlist:elvwasm::QList = serde_json::from_str(std::str::from_utf8(&res).unwrap()).unwrap();
    ///   // do stuff with the qlist
    ///   Ok(res)
    /// }
    /// ```
    pub fn q_list_content_for(&'a self, qlibid:&str) -> CallResult {

      let j = json!(
        {
          "external_lib" : qlibid,
        }
      );

      self.call_function("QListContentFor", j, "core")
    }

    /// q_list_content calculates a content fabric QList for the library assoictaed with this bitcode
    /// # Arguments
    /// # Returns
    /// [Vec<u8>] parseable to [QList]
    /// e.g.
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   let res = bcc.q_list_content()?;
    ///   let qlist:elvwasm::QList = serde_json::from_str(std::str::from_utf8(&res).unwrap()).unwrap();
    ///   // do stuff with the qlist
    ///   Ok(res)
    /// }
    /// ```
    pub fn q_list_content(&'a self) -> CallResult{
      let j = json!({});
      self.call_function("QListContent", j, "core")
    }


    pub fn make_success(&'a self, msg:&str) -> CallResult {
      let js_ret = json!({"jpc":"1.0", "id": self.request.id, "result" : msg});
      let v = serde_json::to_vec(&js_ret)?;
      let out = std::str::from_utf8(&v)?;
      elv_console_log(&format!("returning : {}", out));
      Ok(v)
    }

    pub fn make_success_json(&'a self, msg:&serde_json::Value, id:&str) -> CallResult {
      let js_ret = json!({
        "result" : msg,
        "jpc" : "1.0",
        "id"  : id,
      });
      let v = serde_json::to_vec(&js_ret)?;
      let out = std::str::from_utf8(&v)?;
      elv_console_log(&format!("returning : {}", out));
      Ok(v)
    }

    pub fn make_error(&'a self, msg:&str) -> CallResult {
      make_json_error(ElvError::<NoSubError>::new(msg , ErrorKinds::Invalid))
    }

    pub fn make_error_with_kind(&'a self, msg:&str, kind:ErrorKinds) -> CallResult {
      make_json_error(ElvError::<NoSubError>::new(msg , kind))
    }

    pub fn make_error_with_error<T:Error>(&'a self, msg:&str, kind:ErrorKinds, err:T) -> CallResult {
      make_json_error(ElvError::<T>::new_with_err(msg , kind, err))
    }


    pub fn make_success_bytes(&'a self, msg:&[u8], id:&str) -> CallResult {
      let res:serde_json::Value = serde_json::from_slice(msg)?;
      let js_ret = json!({"jpc":"1.0", "id": id, "result" : res});
      let v = serde_json::to_vec(&js_ret)?;
      Ok(v)
    }

    /// The following sqmd_* based functions all work on the premise that a bitcode context represents a single content
    /// in the fabric.  As such, each content has meta data that is directly associated.  This meta forms a standard tree
    /// at the `/` root level.

    /// sqmd_get_json gets the metadata at path
    /// # Arguments
    /// * `path` : path to the meta data
    /// # Returns
    /// * UTF8 [u8] slice containing json
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   let res = bcc.sqmd_get_json("/some_key")?;
    ///   let mut meta_str = String::from_utf8(res.clone());
    ///   Ok(res)
    /// }
    /// ```
    pub fn sqmd_get_json(&'a self, path:&'a str) -> CallResult {
      let sqmd_get = json!
      (
        {
          "path": path
        }
      );
      self.call_function("SQMDGet", sqmd_get, "core")
    }

    /// sqmd_get_json_external gets the metadata at path from another content
    /// # Arguments
    /// * `path` : path to the meta data
    /// * `qhash`: hash of external content
    /// # Returns
    /// * UTF8 [u8] slice containing json
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   let res = bcc.sqmd_get_json_external("libid4556", "hq_bad2a1ac0a2923ad85e1489736701c06320242a9", "/some_key")?;
    ///   let mut meta_str: String = String::from_utf8(res.clone())?;
    ///   Ok(res)
    /// }
    /// ```
    pub fn sqmd_get_json_external(&'a self, qlibid:&str, qhash:&str, path:&str) -> CallResult {
      let sqmd_get = json!
      (
        {
          "path": path,
          "qlibid":qlibid,
          "qhash":qhash,
        }
      );
      self.call_function("SQMDGetExternal", sqmd_get, "core")
    }

    /// sqmd_clear_json clears the metadata at path
    /// # Arguments
    /// * `path` : path to the meta data
    /// # Returns
    /// * nothing only error on failure
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   let res = bcc.sqmd_clear_json("/some_key")?; // this will blast some_key and all its descendants
    ///   Ok(res)
    /// }
    /// ```
    pub fn sqmd_clear_json(&'a self, path:&'a str) -> CallResult {

      let sqmd_clear = json!
      (
        {
          "path": path,
        }
      );
      self.call_function("SQMDClear", sqmd_clear, "core")
    }

    pub fn sqmd_delete_json(&'a self, token:&'a str, path:&'a str) -> CallResult {

      let sqmd_delete = json!
      (
        {
          "token":token,
          "path": path,
        }
      );
      self.call_function("SQMDDelete", sqmd_delete, "core")
    }

    pub fn sqmd_set_json(&'a self, path:&'a str, json_str:&'a str) -> CallResult {

      let sqmd_set = json!
      (
        {
          "meta":json_str,
          "path": path,
        }
      );
      self.call_function("SQMDSet", sqmd_set, "core")
    }

    /// sqmd_merge_json
    pub fn sqmd_merge_json(&'a self, path:&'a str, json_str:&'a str) -> CallResult {

      let sqmd_merge = json!
      (
        {
          "meta":json_str,
          "path": path,
        }
      );
      self.call_function("SQMDMerge", sqmd_merge, "core")
    }
    /// proxy_http proxies an http request in case of CORS issues
    /// # Arguments
    /// * `v` : a JSON Value
    ///
    /// ```
    ///fn do_something<'s, 'r>(bcc: &'s elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   let v = serde_json::from_str(r#"{
    ///         "request_parameters" : {
      ///         "url": "https://www.googleapis.com/customsearch/v1?key=AIzaSyCppaD53DdPEetzJugaHc2wW57hG0Y5YWE&q=fabric&cx=012842113009817296384:qjezbmwk0cx",
    ///         "method": "GET",
    ///         "headers": {
    ///         "Accept": "application/json",
    ///         "Content-Type": "application/json"
    ///       }
    ///   }"#).unwrap();
    ///   bcc.proxy_http(v)
    /// }
    /// ```
    /// # Returns
    /// * slice of [u8]
    pub fn proxy_http(&'a self, v:serde_json::Value) -> CallResult {
      let method = "ProxyHttp";
      let proxy_result = self.call_function(method, v, "ext")?;
      let id = self.request.id.clone();
      self.make_success_bytes(&proxy_result, &id)
    }

    pub fn call(&'a mut self, ns: &str, op: &str, msg: &[u8]) -> CallResult{
      host_call(self.request.id.as_str(),ns,op,msg)
    }
    /// call_function - enables the calling of fabric api's
    /// # Arguments
    /// * `fn_name` - the fabric api to call e.g. QCreateFileFromStream
    /// * `params` - a json block to pass as parameters to the function being called
    /// * `module` - one of {"core", "ctx", "ext"} see [fabric API]
    ///
    pub fn call_function(&'a self, fn_name:&str , params:serde_json::Value, module:&str) -> CallResult {
      let response = &Response{
        jpc:"1.0".to_string(),
        id:self.request.id.clone(),
        module:module.to_string(),
        method : fn_name.to_string(),
        params,
      };
      let call_val = serde_json::to_vec(response)?;
      let call_str = serde_json::to_string(response)?;

      elv_console_log(&format!("CALL STRING = {}", call_str));
      host_call(self.request.id.as_str(),module, fn_name, &call_val)
    }

    /// close_stream closes the fabric stream
    /// - sid:    the sream id (returned from one of the new_file_stream or new_stream)
    ///  Returns the checksum as hex-encoded string
    pub fn close_stream(&'a self, sid : String) -> CallResult{
      self.call_function("CloseStream", serde_json::Value::String(sid), "ctx")
    }

    /// new_stream creates a new fabric bitcode stream.
    /// # Returns
    /// * output [u8] of format `{"stream_id" : id}` where id is a string
    pub fn new_stream(&'a self) -> CallResult {
      let v = json!({});
      self.call_function("NewStream", v, "ctx")
    }

    /// new_file_stream creates a new fabric file
    /// output [u8] of format where id and path are strings
    /// {
    ///   "stream_id": id,
    ///   "file_name": path
    /// }
    pub fn new_file_stream(&'a self) -> CallResult {
      let v = json!({});
      self.call_function("NewFileStream", v, "ctx")
    }

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
    pub fn ffmpeg_run(&'a self, cmdline:Vec<&str>) -> CallResult {
      let params = json!({
        "stream_params" : cmdline
      });
      self.call_function( "FFMPEGRun", params, "ext")
    }

    /// q_download_file : downloads the file stored  at the fabric file location path for some content
    /// # Arguments
    /// *  `path` : fabric file location in the content
    /// *  `hash_or_token` : hash for the content containing the file
    ///
    pub fn q_download_file(&'a mut self, path:&str, hash_or_token:&str) -> CallResult{
      elv_console_log(&format!("q_download_file path={} token={}",path,hash_or_token));
      let strm = self.new_stream()?;
      let strm_json:serde_json::Value = serde_json::from_slice(&strm)?;
      let sid = strm_json["stream_id"].to_string();
      if sid.is_empty(){
        return self.make_error("Unable to find stream_id");
      }
      let j = json!({
        "stream_id" : sid,
        "path" : path,
        "hash_or_token": hash_or_token,
      });

      let ret = self.call_function("QFileToStream", j, "core");
      let v:serde_json::Value;
      match ret{
        Err(e) => return Err(e),
        Ok(e) => v = serde_json::from_slice(&e).unwrap_or_default()
      }

      let jtemp = v.to_string();
      elv_console_log(&format!("json={}", jtemp));
      let written = v["written"].as_u64().unwrap_or_default();

      if written != 0 {
        return self.read_stream(sid, written as usize);
      }
      self.make_error("failed to write data")

    }

    /// q_upload_file : uploads the input data and stores it at the fabric file location as filetype mime
    /// # Arguments
    /// * `qwt` : a fabric write token
    /// *  `input_data` : a slice of u8 data
    /// *  `path` : fabric file location
    /// *  `mime` : MIME type to store the data as (eg gif)
    ///
    pub fn q_upload_file(&'a mut self, qwt:&str, input_data:&[u8], path:&str, mime:&str) -> CallResult{
      let sid = self.new_file_stream()?;
      let new_stream:FileStream = serde_json::from_slice(&sid)?;
      defer!{
        let _ = self.close_stream(new_stream.stream_id.clone());
      }
      let ret_s = self.write_stream(qwt, new_stream.clone().stream_id.as_str(), input_data, input_data.len())?;
      let written_map:HashMap<String, String> = serde_json::from_slice(&ret_s)?;
      let i: i32 = written_map["written"].parse().unwrap_or(0);
      let j = json!({
        "qwtoken" : qwt,
        "stream_id": new_stream.stream_id,
        "path":path,
        "mime":mime,
        "size": i,
      });

      let method = "QCreateFileFromStream";
      self.call_function(method, j, "core")
    }

    /// file_to_stream directs a fabric file (filename) to a fabric stream (stream)
    /// filename - name of the fabric file (see new_file_stream)
    /// stream - name of the stream that receives the file stream (see new_stream)
    pub fn file_to_stream(&'a self, filename:&str, stream:&str) -> CallResult {
      let param = json!({ "stream_id" : stream, "path" : filename});
      self.call_function("FileToStream", param, "core")
    }

    /// file_stream_size computes the current size of a fabric file stream given its stream name
    ///     filename : the name of the file steam.  See new_file_stream.
    pub fn file_stream_size(&'a self,filename:&str) -> usize {
      elv_console_log("file_stream_size");
      let ret:Vec<u8> = match self.call_function("FileStreamSize", json!({"file_name" : filename}), "ctx"){
          Ok(m) =>{ m }
          Err(_e) => {
            let j:FileStreamSize = serde_json::from_value(json!({"file_size" : 0})).unwrap_or(FileStreamSize{file_size:0});
            return j.file_size;
          }
      };

      match serde_json::from_slice::<FileStreamSize>(&ret){
        Ok(msize) => {
          elv_console_log(&format!("FileStream returned={}", msize.file_size));
          msize.file_size
        }
        Err(_e) => {
          elv_console_log("Err from FileStreamSize");
          0
        }
      }
    }

  }

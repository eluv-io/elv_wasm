use serde::{Deserialize, Serialize};
use serde_json::json;

extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate wapc_guest as guest;
extern crate thiserror;

use thiserror::Error;
use std::fmt::Debug;

use std::str;
use std::collections::HashMap;


use guest::prelude::*;
use guest::CallResult;

macro_rules! implement_ext_func {
  (
    $(#[$meta:meta])*
    $handler_name:ident,
    $fabric_name:literal
  ) => {
    $(#[$meta])*
    pub fn $handler_name(&'a self, v:serde_json::Value) -> CallResult {
      let method = $fabric_name;
      let impl_result = self.call_function(method, v, "ext")?;
      let id = self.request.id.clone();
      self.make_success_bytes(&impl_result, &id)
    }
  }
}

#[derive(Error, Debug, Clone, Serialize, Copy)]
pub enum ErrorKinds {
  #[error("Other Error : `{0}`")]
  Other(&'static str),
  #[error("NotImplemented : `{0}`")]
  NotImplemented(&'static str),
  #[error("Invalid : `{0}`")]
  Invalid(&'static str),
  #[error("Permission : `{0}`")]
  Permission(&'static str),
  #[error("IO : `{0}`")]
  IO(&'static str),
  #[error("Exist : `{0}`")]
  Exist(&'static str),
  #[error("NotExist : `{0}`")]
  NotExist(&'static str),
  #[error("IsDir : `{0}`")]
  IsDir(&'static str),
  #[error("NotDir : `{0}`")]
  NotDir(&'static str),
  #[error("Finalized : `{0}`")]
  Finalized(&'static str),
  #[error("NotFinalized : `{0}`")]
  NotFinalized(&'static str),
  #[error("BadHttpParams : `{0}`")]
  BadHttpParams(&'static str),
}

#[cfg(not(test))]
fn elv_console_log(s:&str){
  console_log(s)
}

#[cfg(test)]
fn elv_console_log(s:&str){
  println!("{}", s)
}

/// make_json_error translates the bitcode [ErrorKinds] to an error response to the client
/// # Arguments
/// * `err`- the error to be translated to a response
pub fn make_json_error(err:ErrorKinds, id:&str) -> CallResult {
    elv_console_log(&format!("error={}", err));
    let msg = json!(
      {
        "error" :  err,
        "jpc" : "1.0",
        "id"  : id,
      }
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
    let s:String = if !self.write_token.is_empty() {
      self.write_token.to_string()
    }else{
      self.hash.to_string()
    };
    s
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QPart {
  #[serde(default)]
	pub write_token:String,
  #[serde(default)]
	pub hash:String,
  #[serde(default)]
	pub size:i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QPartInfo {
	pub content: Q,
  pub part: QPart,
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

/// Bitcode representation of a result from new_stream
/// ```
/// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
///   let res = bcc.new_stream()?;
///   let stream1:elvwasm::NewStreamResult = serde_json::from_slice(&res)?;
///   // stream1.stream_id has new id
///   Ok(res)
/// }
/// ```
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewStreamResult {
	pub stream_id: String,
}

/// Bitcode representation of a result from read_stream
/// ```
/// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
///   let res = bcc.new_stream()?;
///   let stream1:elvwasm::NewStreamResult = serde_json::from_slice(&res)?;
///   // stream1.stream_id has new id
///   Ok(res)
/// }
/// ```
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReadStreamResult {
  #[serde(rename = "return")]
  pub retval: String,
  pub result: String
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

    pub fn log(s: &str) {
      console_log(s);
    }

    // CORE functions


    /// q_create_content creates a new content object locally.  The content will have a write token but will
    /// not be comitted to the fabric until a calls to Finalize and commit are made
    /// # Arguments
    /// * `qtype`-   a hash for the content type. Can also be "builtin" for built in bitcode
    /// * `meta`-    a HashMap containing the initial meta data for the object to be set at '/'
    /// # Returns
    /// utf8 bytes stream containing json
    /// { "qid" : "idObj", "qwtoken" : "writeToken"}
    pub fn q_create_content (&'a self, qtype:&str, meta:&HashMap<&str, serde_json::Value>) -> CallResult {
      let msg = json!({
        "qtype" : qtype,
        "meta"  : meta,
      });
      self.call_function("QCreateContent", msg, "core")
    }


    /// q_list_content calculates a content fabric QList for the context's libid
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
    pub fn q_list_content(&'a self) -> CallResult {
      self.call_function("QListContent", json!({}), "core")
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

    /// q_finalize_content finalizes a given write token
    /// # Arguments
    /// * `qwtoken` - a write token to finalize
    /// # Returns
    /// utf8 bytes stream containing json
    /// { "qid" : "idObj", "qhash" : "newHash"}
    /// e.g.
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   let res = bcc.q_list_content()?;
    ///   let q:serde_json::Map = serde_json::from_str(std::str::from_utf8(&res).unwrap()).unwrap();
    ///   let id:&str = q["qid"];
    ///   let hash:&str = q["qhash"];
    ///   Ok(res)
    /// }
    /// ```
    pub fn q_finalize_content(&'a self, qwtoken:&str) -> CallResult {
      let msg = json!(
        {
          "qwtoken" : qwtoken,
        }
      );
      self.call_function("QFinalizeContent", msg, "core")
    }

    /// q_commit_content finalizes a given write token
    /// # Arguments
    /// * `qhash` - a finalized hash
    /// # Returns
    /// nil
    /// e.g.
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   let res = bcc.q_commit_content("hq__jd7sd655fffg7HrF76mHDolzzwe")?;
    ///   Ok("SUCCESS".to_owned().as_bytes().to_vec())
    /// }
    /// ```
    pub fn q_commit_content(&'a self, qhash:&str) -> CallResult {
      let msg = json!(
        {
          "qhash" : qhash,
        }
      );
      self.call_function("QFinalizeContent", msg, "core")
    }

    /// q_modify_content enables edit on the implicit content of the context
    /// # Returns
    /// utf8 bytes stream containing json
    /// { "qwtoken" : "writeTokenForEdit"}
    /// e.g.
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   let res = bcc.q_modify_content()?;
    ///   let q:serde_json::Map = serde_json::from_str(std::str::from_utf8(&res).unwrap())?;
    ///   let write_token:&str = q["qwtoken"];
    ///   Ok(res)
    /// }
    /// ```
    pub fn q_modify_content(&'a self) -> CallResult {
      self.call_function("QModifyContent", json!({}), "core")
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

    /// q_create_part_from_stream creates a new part in a writeable object from a context stream.
    /// The content will be made locally but not published until finalized and committed
    /// # Arguments
    /// * `qwtoken`-   a write token to write the part
    /// * `stream_id`- the stream to write [BitcodeContext::new_stream]
    /// # Returns
    /// utf8 bytes stream containing json
    /// { "qphash" : "newPartHash", "size" : SizeOfPart}
    pub fn q_create_part_from_stream (&'a self, qwtoken:&str, stream_id:&str) -> CallResult {
      let msg = json!({
        "qwtoken" : qwtoken,
        "stream_id"  : stream_id,
      });
      self.call_function("QCreatePartFromStream", msg, "core")
    }

    /// q_file_to_stream writes the given qfile's content to the provided stream
    /// # Arguments
    /// * `stream_id`-  string identifier aquired from [BitcodeContext::new_stream]
    /// * `path` - string conatining the QFile path
    /// * `hash_or_token` - an optional string with a hash to operate on (defaults to the contexts content)
    /// # Returns
    /// [Vec<u8>] with undelying json
    /// {"written", written}
    pub fn q_file_to_stream(&'a self, stream_id:&str, path:&str, hash_or_token:&str) -> CallResult {
      let j = json!(
        {
          "stream_id" : stream_id,
          "path" : path,
          "hash_or_token" : hash_or_token
        }
      );

      self.call_function("QFileToStream", j, "core")
    }

    /// file_to_stream directs a fabric file (filename) to a fabric stream (stream)
    /// filename - name of the fabric file (see new_file_stream)
    /// stream - name of the stream that receives the file stream (see new_stream)
    pub fn file_to_stream(&'a self, filename:&str, stream:&str) -> CallResult {
      let param = json!({ "stream_id" : stream, "path" : filename});
      self.call_function("FileToStream", param, "core")
    }


    /// q_create_file_from_stream creates a qfile from the cotents of a bitcode stream
    /// # Arguments
    /// * `stream_id`-    stream identifier from new_stream or the like
    /// * `qwtoken`-  write token (will use context's if "" is provided)
    /// * `path`-  qfile path
    /// * `mime` - MIME type of the file
    /// * `size` - size of the file in bytes
    /// # Returns
    /// [Vec<u8>] parseable to [QPartInfo]
    pub fn q_create_file_from_stream(&'a self, stream_id:&str, qwtoken:&str, path:&str, mime:&str, size:i64) -> CallResult{
      let msg = json!(
        {
          "stream_id" :  stream_id,
          "qwtoken":qwtoken,
          "path" : path,
          "mime": mime,
          "size": size,
       }
      );
      self.call_function("QCreateFileFromStream", msg, "core")
    }

    /// q_create_q_state_store creates a new state store in the fabric
    /// # Returns
    /// utf8 bytes stream containing a string with the state store id
    /// e.g.
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   let res = bcc.q_create_q_state_store()?;
    ///   let ssID = std::str::from_utf8(&res)?;
    ///   Ok(res)
    /// }
    /// ```
    pub fn q_create_q_state_store(&'a self) -> CallResult {
      self.call_function("QCreateQStateStore", json!({}), "core")
    }

    /// q_checksum_part calculates a checksum of a given content part.
    /// # Arguments
    /// * `sum_method`-    checksum method ("MD5" or "SHA256")
    /// * `qphash`-        hash of the content part to checksum
    /// # Returns
    /// the checksum as hex-encoded string
    pub fn q_checksum_part(&'a self, sum_method:&str, qphash:&str) -> CallResult{

      let j = json!(
        {
          "method" : sum_method,
          "qphash" : qphash
        }
      );

      self.call_function("QCheckSumPart", j, "core")
    }


    /// q_checksum_file calculates a checksum of a file in a file bundle
    /// # Arguments
    /// * `sum_method`-    checksum method ("MD5" or "SHA256")
    /// * `file_path`-     the path of the file in the bundle
    /// # Returns
    /// the checksum as hex-encoded string
    pub fn q_checksum_file(&'a self, sum_method:&str, file_path:&str) -> CallResult{

      let j = json!(
        {
          "method" : sum_method,
          "file_path" : file_path,
        }
      );

      self.call_function("QCheckSumFile", j, "core")
    }

    /// The following sqmd_* based functions all work on the premise that a bitcode context represents a single content
    /// in the fabric.  As such, each content has meta data that is directly associated.  This meta forms a standard tree
    /// at the `/` root level.
    ///

    /// sqmd_set_json ets the metadata at path
    /// # Arguments
    /// * `path` : path to the meta data
    /// * `val` : serde_json::Value to set
    /// # Returns
    /// * error only no success return
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   bcc.sqmd_set_json("/some_key", json!({"foo" : "bar"}))?;
    ///   Ok("SUCCESS".to_owned().as_bytes().to_vec())
    /// }
    /// ```
    pub fn sqmd_set_json(&'a self, path:&'a str, val:&serde_json::Value) -> CallResult {

      let sqmd_set = json!
      (
        {
          "meta": val,
          "path": path,
        }
      );
      self.call_function("SQMDSet", sqmd_set, "core")
    }

    /// sqmd_merge_json merges the metadata at path
    /// # Arguments
    /// * `path` : path to the meta data
    /// * `val` : serde_json::Value to merge
    /// # Returns
    /// * error only no success return
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   bcc.sqmd_merge_json("/some_key", json!({"foo" : "bar"}))?;
    ///   Ok("SUCCESS".to_owned().as_bytes().to_vec())
    /// }
    /// ```
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

    /// sqmd_delete_json deletes the metadata at path
    /// # Arguments
    /// * `path` : path to the meta data
    /// # Returns
    /// * error only no success return
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   bcc.sqmd_delete_json("/some_key")?;
    ///   Ok("SUCCESS".to_owned().as_bytes().to_vec())
    /// }
    /// ```
    pub fn sqmd_delete_json(&'a self, path:&'a str) -> CallResult {

      let sqmd_delete = json!
      (
        {
          "path": path,
        }
      );
      self.call_function("SQMDDelete", sqmd_delete, "core")
    }

    /// sqmd_clear_json clears the metadata at path
    /// # Arguments
    /// * `path` : path to the meta data
    /// # Returns
    /// * nothing only error on failure
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   bcc.sqmd_clear_json("/some_key")?; // this will blast some_key and all its descendants
    ///   Ok("SUCCESS".to_owned().as_bytes().to_vec())
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

    /// sqmd_get_json gets the metadata at path
    /// # Arguments
    /// * `path` : path to the meta data
    /// # Returns
    /// * UTF8 [u8] slice containing json
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   let res = bcc.sqmd_get_json("/some_key")?;
    ///   let mut meta = serde_json::from_utf8(res.clone());
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

    /// sqmd_get_json_resolve gets the metadata at path resolving all links
    /// # Arguments
    /// * `path` : path to the meta data
    /// # Returns
    /// * UTF8 [u8] slice containing json
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   let res = bcc.sqmd_get_json_resolve("/some_key")?;
    ///   let mut meta = serde_json::from_utf8(res.clone());
    ///   Ok(res)
    /// }
    /// ```
    pub fn sqmd_get_json_resolve(&'a self, path:&'a str) -> CallResult {
      let sqmd_get = json!
      (
        {
          "path": path
        }
      );
      self.call_function("SQMDGetJSONResolve", sqmd_get, "core")
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

    //
    /// sqmd_query queries the meta-data with the given JSONPath query expression.
    /// # Arguments
    /// * `query` : JSONPath query expression
    /// # Returns
    /// * UTF8 [u8] slice containing json
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   let res = bcc.sqmd_query("$['some'].value[0].description")?;
    ///   let mut meta = serde_json::from_utf8(res.clone());
    ///   Ok(res)
    /// }
    /// ```
    pub fn sqmd_query(&'a self, query:&'a str) -> CallResult {
      let sqmd_query = json!
      (
        {
          "query": query
        }
      );
      self.call_function("SQMDQuery", sqmd_query, "core")
    }

    /// qss_set sets data into the Q state store
    /// # Arguments
    /// * `qssid`- string identifier aquired from [BitcodeContext::q_create_q_state_store]
    /// * `key` - string
    /// * `val` - string value to store
    /// # Returns
    /// Nothing error only
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   let res = bcc.q_create_q_state_store()?;
    ///   let ssID = std::str::from_utf8(&res)?;
    ///   bcc.qss_set(ssID, "akey", "avalue")?;
    ///   Ok("SUCCESS".to_owned().as_bytes().to_vec())
    /// }
    /// ```
    pub fn qss_set(&'a self, qssid:&str, key:&str, val:&str) -> CallResult {
      let j = json!(
        {
          "qssid" : qssid,
          "key" : key,
          "val" : val
        }
      );

      self.call_function("QSSSet", j, "core")
    }

    /// qss_get gets data from the Q state store
    /// # Arguments
    /// * `qssid`- string identifier aquired from [BitcodeContext::q_create_q_state_store]
    /// * `key` - string
    /// # Returns
    /// [Vec<u8>] containing string value
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   let res = bcc.qss_get("sid_648nfjfh5666nmjejh", "akey")?;
    ///   let strVal = std::str::from_utf8(&res)?;
    ///   Ok(res)
    /// }
    /// ```
    pub fn qss_get(&'a self, qssid:&str, key:&str) -> CallResult {
      let j = json!(
        {
          "qssid" : qssid,
          "key" : key,
        }
      );

      self.call_function("QSSGet", j, "core")
    }

    /// qss_get deletes data from the Q state store
    /// # Arguments
    /// * `qssid`- string identifier aquired from [BitcodeContext::q_create_q_state_store]
    /// * `key` - string
    /// # Returns
    /// Nothing error only
    /// ```
    /// fn do_something<'s, 'r>(bcc: &'s mut elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
    ///   bcc.qss_delete("sid_648nfjfh5666nmjejh", "akey")?;
    ///   Ok("SUCCESS".to_owned().as_bytes().to_vec())
    /// }
    /// ```
    pub fn qss_delete(&'a self, qssid:&str, key:&str) -> CallResult {
      let j = json!(
        {
          "qssid" : qssid,
          "key" : key,
        }
      );

      self.call_function("QSSDelete", j, "core")
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
    pub fn write_stream(&'a self, stream:&str,  src:&'a [u8], len: usize) -> CallResult {
      let mut actual_len = src.len();
      if len != usize::MAX {
        actual_len = len
      }
      let v = serde_json::json!(src[..actual_len]);
      let jv = &serde_json::to_vec(&v)?;
      host_call(&self.request.id, stream, "Write", jv)
    }

    /// write_stream writes a u8 slice to a fabric stream
    /// # Arguments
    /// * `id`-    a unique identifier (can use BitcodeContext's request id)
    /// * `stream`-  the fabric stream to write to [BitcodeContext::new_stream]
    /// * `src`-  a u8 slice to write
    /// # Returns
    /// utf8 bytes stream containing json
    /// { "written" : bytes }
    pub fn write_stream_auto(id:String, stream:&'a str,  src:&'a [u8]) -> CallResult {
      host_call(&id, stream, "Write", src)
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
    pub fn read_stream(&'a self, stream_to_read:String, sz:usize) -> CallResult {
          let input = vec![0; sz];
          host_call(self.request.id.as_str(),stream_to_read.as_str(), "Read", &input)
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

    pub fn make_error(&'a self, msg:&'static str) -> CallResult {
      make_json_error(ErrorKinds::Invalid(msg), &self.request.id)
    }

    pub fn make_error_with_kind(&'a self, kind:ErrorKinds) -> CallResult {
      make_json_error(kind, &self.request.id)
    }

    pub fn make_error_with_error<T:>(&'a self, kind:ErrorKinds, _err:T) -> CallResult {
      make_json_error(kind, &self.request.id)
    }


    pub fn make_success_bytes(&'a self, msg:&[u8], id:&str) -> CallResult {
      let res:serde_json::Value = serde_json::from_slice(msg)?;
      let js_ret = json!({"jpc":"1.0", "id": id, "result" : res});
      let v = serde_json::to_vec(&js_ret)?;
      Ok(v)
    }


    implement_ext_func!(
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
      proxy_http, "ProxyHttp"
    );


    implement_ext_func!(
      /// new_index_builder create a new Tantivy index builder
      new_index_builder, "NewIndexBuilder"
    );

    implement_ext_func!(
      /// builder_add_text_field adds a new text field to a Tantivy index
      builder_add_text_field, "BuilderAddTextField"
    );
    implement_ext_func!(
      /// builder_build builds the new Index
      builder_build, "BuilderBuild"
    );

    implement_ext_func!(
      /// document_create create a new document for a given Index
      document_create, "DocumentCreate"
    );

    implement_ext_func!(
      /// document_add_text add text to a given document
      document_add_text, "DocumentAddText"
    );

    implement_ext_func!(
      /// document_create_index creates an index given a set of documents
      document_create_index, "DocumentCreateIndex"
    );

    implement_ext_func!(
      /// index_create_writer creates an index writer
      index_create_writer, "IndexCreateWriter"
    );

    implement_ext_func!(
      /// index_add_document adds a document to the writer
      index_add_document, "IndexWriterAddDocument"
    );

    implement_ext_func!(
      /// index_writer_commit commits the index
      index_writer_commit, "IndexWriterCommit"
    );

    implement_ext_func!(
      /// index_reader_builder_create creates a new reader builder on an index
      index_reader_builder_create, "IndexReaderBuilderCreate"
    );

    implement_ext_func!(
      /// reader_builder_query_parser_create creates a ReaderBuilder from a QueryParser
      reader_builder_query_parser_create, "ReaderBuilderQueryParserCreate"
    );

    implement_ext_func!(
      /// query_parser_for_index executes ForIndex on the QueryParser
      /// # Arguments
      /// * `v` : a JSON Value
      /// ```
      /// fn do_something<'s, 'r>(bcc: &'s elvwasm::BitcodeContext<'r>) -> wapc_guest::CallResult {
      ///   let v = serde_json::from_str(r#"{
      ///         "fields" : ["field1", "field2"]
      ///       }
      ///   }"#).unwrap();
      ///   bcc.query_parser_for_index(v)
      /// }
      /// ```
      /// # Returns
      /// * slice of [u8]
    query_parser_for_index, "QueryParserForIndex"
    );

    implement_ext_func!(
      /// query_parser_parse_query parses a given query into the QueryParser to search on
      query_parser_parse_query, "QueryParserParseQuery"
    );

    implement_ext_func!(
      /// query_parser_search searches the given QueryParser for the term
      query_parser_search, "QueryParserSearch"
    );

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
      let call_ret_val = host_call(self.request.id.as_str(),module, fn_name, &call_val)?;
      let j_res:serde_json::Value = serde_json::from_slice(&call_ret_val)?;
      if !j_res.is_object(){
        return Ok(call_ret_val);
      }
      return match j_res.get("result"){
        Some(x) => {
          BitcodeContext::log("here In result");
          let r = serde_json::to_vec(&x)?;
          Ok(r)
        },
        None => {
          match j_res.get("error"){
            Some(x) => {
              BitcodeContext::log("here in error");
              let r = serde_json::to_vec(&x)?;
              return Ok(r);
            },
            None => {
              BitcodeContext::log("here in neither");
              return Ok(call_ret_val);
            }
          };
        }
      };
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
        return self.make_error_with_kind(ErrorKinds::IO("Unable to find stream_id"));
      }
      let j = json!({
        "stream_id" : sid,
        "path" : path,
        "hash_or_token": hash_or_token,
      });

      let v:serde_json::Value = match self.call_function("QFileToStream", j, "core"){
        Err(e) => return Err(e),
        Ok(e) => serde_json::from_slice(&e).unwrap_or_default()
      };

      let jtemp = v.to_string();
      elv_console_log(&format!("json={}", jtemp));
      let written = v["written"].as_u64().unwrap_or_default();

      if written != 0 {
        return self.read_stream(sid, written as usize);
      }
      self.make_error_with_kind(ErrorKinds::NotExist("failed to write data"))

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
      let ret_s = self.write_stream(new_stream.clone().stream_id.as_str(), input_data, input_data.len())?;
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

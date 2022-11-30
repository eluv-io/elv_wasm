extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate thiserror;
extern crate wapc_guest as guest;

use serde_derive::{Deserialize, Serialize};
use std::fmt::Debug;
use std::collections::HashMap;
use std::str;

/// Q is a bitcode representation of an individual piece of content from the fabric
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Q {
    pub id: String,
    pub hash: String,
    #[serde(default)]
    pub write_token: String,
    #[serde(rename = "type")]
    pub q_type: String,
    pub qlib_id: String,
    #[serde(default)]
    pub meta: serde_json::Value,
    #[serde(default)]
    pub size_stats:SizeStats,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SizeStats{
	pub parts:i32,
    #[serde(default)]
	pub size:String,
    pub size_bytes:i64,
}

/// Bitcode representation of a fabric size error
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QError {
    pub error: String,
    #[serde(default)]
    pub item: Q,
}

/// QRef is a bitcode representation of versioned content from the fabric
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct QRef {
    pub id: String,
    pub versions: Vec<Q>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ReadResult {
    pub result: String,
    #[serde(rename = "return", default)]
    pub ret: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct WritePartResult {
    pub written: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SystemTimeResult {
    pub time: u64,
}



/// Bitcode representation of a full content listing given an optional filter
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QList {
    #[serde(default)]
    pub filter: String,
    pub contents: Vec<QRef>,
    #[serde(default)]
    pub errors: Vec<QError>,
}

/// Bitcode representation of a fabric FileStream
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileStream {
    /// bitcode stream handle
    pub stream_id: String,
    /// fabric file path
    pub file_name: String,
}

/// Bitcode representation of the size of a fabric stream
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileStreamSize {
    pub file_size: usize,
}

/// Bitcode representation of the JPC (JSON Procedure Call) parameters from a client request
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct JpcParams {
    pub http: HttpParams,
}

/// Bitcode representation of the http parameters from a client request
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
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
    pub content_length: usize,
    #[serde(default)]
    pub client_ip: String,
    #[serde(default)]
    pub self_url: String,
    #[serde(default)]
    pub proto: String,
    #[serde(default)]
    pub host: String,
}

/// Bitcode representation of a content sans meta data
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
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
    pub fn qhot(&self) -> String {
        let s: String = if !self.write_token.is_empty() {
            self.write_token.to_string()
        } else {
            self.hash.to_string()
        };
        s
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QPart {
    #[serde(default)]
    pub write_token: String,
    #[serde(default)]
    pub hash: String,
    #[serde(default)]
    pub size: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QPartListContents {
    #[serde(default)]
    pub content: Q,
    #[serde(default)]
    pub parts: Vec<QPart>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QPartList {
    pub part_list: QPartListContents
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QPartInfo {
    pub content: Q,
    pub part: QPart,
}

/// Bitcode representation of a incomming client request
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
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
    pub params: serde_json::Value,
    pub id: String,
    pub module: String,
    pub method: String,
}

/// Bitcode representation of a result from new_stream
/// ```
/// fn do_something<'s>(bcc: &'s mut elvwasm::BitcodeContext) -> wapc_guest::CallResult {
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
/// fn do_something<'s>(bcc: &'s mut elvwasm::BitcodeContext) -> wapc_guest::CallResult {
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
    pub result: String,
}

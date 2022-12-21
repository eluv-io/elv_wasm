extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate thiserror;
extern crate wapc_guest as guest;

use crate::{Request, Response, FileStream, FileStreamSize};
use crate::{elv_console_log, make_json_error, ErrorKinds};

use serde_json::json;

use std::fmt::Debug;

use std::collections::HashMap;
use std::str;

use guest::prelude::*;
use guest::CallResult;

/// This structure encapsulates all communication with the Eluvio content fabric.  A new BitcodeContext
/// is automatically created during the processing of the http request.  During initialization, all context
/// data is acquired from the http request.  The BitcodeContext provides 2 way communication to the content fabric.
/// There is convenience impl method [BitcodeContext::call_function] that allows the fabric to be accessed via a known set of APIs.
#[derive(Debug, Clone, Default)]
pub struct BitcodeContext {
    pub request: Request,
    pub return_buffer: Vec<u8>,
    pub index_temp_dir: Option<String>,
}

impl<'a> BitcodeContext {
    pub fn new(request: Request) -> BitcodeContext {
        BitcodeContext {
            request,
            return_buffer: vec![],
            index_temp_dir: None,
        }
    }


    pub fn log(s: &str){
        console_log(s);
    }

    pub fn log_info(&'a self, s: &str) -> CallResult{
        self.call_function("Log", json!({"level" : "INFO", "msg" : s}), "ctx")
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
    pub fn write_stream(&'a self, stream: &str, src: &'a [u8], len: usize) -> CallResult {
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
    pub fn write_stream_auto(id: String, stream: &'a str, src: &'a [u8]) -> CallResult {
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
    pub fn read_stream(&'a self, stream_to_read: String, sz: usize) -> CallResult {
        let input = vec![0; sz];
        BitcodeContext::log(&format!("imput len = {}", input.len()));
        host_call(
            self.request.id.as_str(),
            stream_to_read.as_str(),
            "Read",
            &input,
        )
    }

    /// callback issues a Callback on the fabric setting up an expectation that the output stream
    /// contains a specified sized buffer
    /// # Arguments
    /// * `status`-    the http status of the call
    /// * `content-type`-     output buffer contents
    /// * `size`-  size of the output contents
    /// # Returns
    /// the checksum as hex-encoded string
    pub fn callback(&'a self, status: usize, content_type: &str, size: usize) -> CallResult {
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
        let method = "Callback";
        self.call_function(method, v, "ctx")
    }

    pub fn make_success(&'a self, msg: &str) -> CallResult {
        let js_ret = json!({"jpc":"1.0", "id": self.request.id, "result" : msg});
        let v = serde_json::to_vec(&js_ret)?;
        let out = std::str::from_utf8(&v)?;
        elv_console_log(&format!("returning : {out}"));
        Ok(v)
    }

    pub fn make_success_json(&'a self, msg: &serde_json::Value, id: &str) -> CallResult {
        let js_ret = json!({
          "result" : msg,
          "jpc" : "1.0",
          "id"  : id,
        });
        let v = serde_json::to_vec(&js_ret)?;
        let out = std::str::from_utf8(&v)?;
        elv_console_log(&format!("returning : {out}"));
        Ok(v)
    }

    pub fn make_error(&'a self, msg: &'static str) -> CallResult {
        make_json_error(ErrorKinds::Invalid(msg.to_string()), &self.request.id)
    }

    pub fn make_error_with_kind(&'a self, kind: ErrorKinds) -> CallResult {
        make_json_error(kind, &self.request.id)
    }

    pub fn make_error_with_error<T>(&'a self, kind: ErrorKinds, _err: T) -> CallResult {
        make_json_error(kind, &self.request.id)
    }

    pub fn make_success_bytes(&'a self, msg: &[u8], id: &str) -> CallResult {
        let res: serde_json::Value = serde_json::from_slice(msg)?;
        let js_ret = json!({"jpc":"1.0", "id": id, "result" : res});
        let v = serde_json::to_vec(&js_ret)?;
        Ok(v)
    }


    pub fn call(&'a self, ns: &str, op: &str, msg: &[u8]) -> CallResult {
        host_call(self.request.id.as_str(), ns, op, msg)
    }
    /// call_function - enables the calling of fabric api's
    /// # Arguments
    /// * `fn_name` - the fabric api to call e.g. QCreateFileFromStream
    /// * `params` - a json block to pass as parameters to the function being called
    /// * `module` - one of {"core", "ctx", "ext"} see [fabric API]
    ///
    pub fn call_function(
        &'a self,
        fn_name: &str,
        params: serde_json::Value,
        module: &str,
    ) -> CallResult {
        let response = &Response {
            jpc: "1.0".to_string(),
            id: self.request.id.clone(),
            module: module.to_string(),
            method: fn_name.to_string(),
            params,
        };
        let call_val = serde_json::to_vec(response)?;
        let call_str = serde_json::to_string(response)?;

        elv_console_log(&format!("CALL STRING = {call_str}"));
        let call_ret_val = host_call(self.request.id.as_str(), module, fn_name, &call_val)?;
        let j_res: serde_json::Value = serde_json::from_slice(&call_ret_val)?;
        if !j_res.is_object() {
            return Ok(call_ret_val);
        }
        return match j_res.get("result") {
            Some(x) => {
                let r = serde_json::to_vec(&x)?;
                Ok(r)
            }
            None => {
                match j_res.get("error") {
                    Some(x) => {
                        let r = serde_json::to_vec(&x)?;
                        return Ok(r);
                    }
                    None => {
                        return Ok(call_ret_val);
                    }
                };
            }
        };
    }

    /// close_stream closes the fabric stream
    /// - sid:    the sream id (returned from one of the new_file_stream or new_stream)
    ///  Returns the checksum as hex-encoded string
    pub fn close_stream(&'a self, sid: String) -> CallResult {
        self.call_function("CloseStream", json!({"stream_id" : sid}), "ctx")
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

    /// q_download_file : downloads the file stored  at the fabric file location path for some content
    /// # Arguments
    /// *  `path` : fabric file location in the content
    /// *  `hash_or_token` : hash for the content containing the file
    ///
    pub fn q_download_file(&'a mut self, path: &str, hash_or_token: &str) -> CallResult {
        elv_console_log(&format!(
            "q_download_file path={} token={}",
            path, hash_or_token
        ));
        let strm = self.new_stream()?;
        let strm_json: serde_json::Value = serde_json::from_slice(&strm)?;
        let sid = strm_json["stream_id"].to_string();
        if sid.is_empty() {
            return self.make_error_with_kind(ErrorKinds::IO(format!("Unable to find stream_id {sid}")));
        }
        let j = json!({
          "stream_id" : sid,
          "path" : path,
          "hash_or_token": hash_or_token,
        });

        let v: serde_json::Value = match self.call_function("QFileToStream", j, "core") {
            Err(e) => return self.make_error_with_kind(ErrorKinds::NotExist(format!("QFileToStream failed path={path}, hot={hash_or_token} sid={sid} e={e}"))),
            Ok(e) => serde_json::from_slice(&e).unwrap_or_default(),
        };

        let jtemp = v.to_string();
        elv_console_log(&format!("json={jtemp}"));
        let written = v["written"].as_u64().unwrap_or_default();

        if written != 0 {
            return self.read_stream(sid, written as usize);
        }
        self.make_error_with_kind(ErrorKinds::NotExist(format!("wrote 0 bytes, sid={sid} path={path}, hot={hash_or_token}")))
    }

    /// q_upload_file : uploads the input data and stores it at the fabric file location as filetype mime
    /// # Arguments
    /// * `qwt` : a fabric write token
    /// *  `input_data` : a slice of u8 data
    /// *  `path` : fabric file location
    /// *  `mime` : MIME type to store the data as (eg gif)
    ///
    pub fn q_upload_file(
        &'a mut self,
        qwt: &str,
        input_data: &[u8],
        path: &str,
        mime: &str,
    ) -> CallResult {
        let sid = self.new_file_stream()?;
        let new_stream: FileStream = serde_json::from_slice(&sid)?;
        defer! {
          let _ = self.close_stream(new_stream.stream_id.clone());
        }
        let ret_s = self.write_stream(
            new_stream.clone().stream_id.as_str(),
            input_data,
            input_data.len(),
        )?;
        let written_map: HashMap<String, String> = serde_json::from_slice(&ret_s)?;
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
    pub fn file_stream_size(&'a self, filename: &str) -> usize {
        elv_console_log("file_stream_size");
        let ret: Vec<u8> =
            match self.call_function("FileStreamSize", json!({ "file_name": filename }), "ctx") {
                Ok(m) => m,
                Err(_e) => {
                    let j: FileStreamSize = serde_json::from_value(json!({"file_size" : 0}))
                        .unwrap_or(FileStreamSize { file_size: 0 });
                    return j.file_size;
                }
            };

        match serde_json::from_slice::<FileStreamSize>(&ret) {
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

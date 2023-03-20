extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate thiserror;
extern crate wapc_guest as guest;

use crate::{elv_console_log, make_json_error, ErrorKinds};
use crate::{FileStream, FileStreamSize, Request, Response};

use serde_json::{json, Value};

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

    // REVIEW: Why does log redirect to the host console_log, but log_info call into (what seems to be) the fabric's logger?
    // Seems like they should either both go to console or both go to fabric
    pub fn log(s: &str) {
        console_log(s);
    }

    pub fn log_info(&'a self, s: &str) -> CallResult {
        self.call_function("Log", json!({"level" : "INFO", "msg" : s}), "ctx")
    }

    // REVIEW: Why does this method exist on the bitcode context? It seems more appropriate to have
    // this just be a general method called `convert_call_result` or something
    pub fn convert<'b, T>(
        &'a self,
        cr: &'b CallResult,
    ) -> Result<T, Box<dyn std::error::Error + Sync + Send>>
    where
        T: serde::Deserialize<'b>,
    {
        match cr {
            Ok(r) => Ok(serde_json::from_slice(r)?),
            Err(e) => Err(Box::new(ErrorKinds::Invalid(e.to_string()))),
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
    /// ```json
    /// { "written" : bytes }
    /// ```
    ///
    /// [Example](https://github.com/eluv-io/elv-wasm/blob/019b88ac27635d5022c2211751f6af5957df2463/samples/external/src/lib.rs#L111)
    ///
    /// REVIEW: It doesn't seems like the len argument actually changes what gets written. I think that this function should be either:
    /// - fixed so that it only writes len bytes out of src (but its still quite clunky, because it takes a slice reference the caller should just slice before writing)
    /// - remove the len argument, and get rid of write_stream_auto as they are now the same thing
    pub fn write_stream(&self, stream: &str, src: &[u8], len: usize) -> CallResult {
        let mut actual_len = src.len();
        if len != usize::MAX {
            actual_len = len
        }
        BitcodeContext::log(&format!("in write_stream len = {actual_len}"));
        host_call(&self.request.id, stream, "Write", src)
    }

    /// write_stream_auto writes a u8 slice to a fabric stream
    /// # Arguments
    /// * `id`-    a unique identifier (can use BitcodeContext's request id)
    /// * `stream`-  the fabric stream to write to [BitcodeContext::new_stream]
    /// * `src`-  a u8 slice to write
    /// # Returns
    /// utf8 bytes stream containing json
    /// ```json
    /// { "written" : bytes }
    /// ```
    ///
    /// [Example](https://github.com/eluv-io/elv-wasm/blob/019b88ac27635d5022c2211751f6af5957df2463/samples/rproxy/src/lib.rs#L31)
    ///
    pub fn write_stream_auto(id: String, stream: &'a str, src: &'a [u8]) -> CallResult {
        host_call(&id, stream, "Write", src)
    }

    /// read_stream reads usize bytes from a fabric stream returning a u8 slice
    /// # Arguments
    /// * `stream_to_read`-  the fabric stream to read from
    /// * `sz`-  usize size of bytes
    /// # Returns
    /// a call result containing a utf8-encoded json bytes in the format
    /// ```json
    /// {
    ///   "return" : { "read" : byte-count-read },
    ///   "result" : "base64 encoded string"
    ///  }
    /// ```
    /// [Example](https://github.com/eluv-io/elv-wasm/blob/019b88ac27635d5022c2211751f6af5957df2463/samples/objtar/src/lib.rs#L112)
    ///
    pub fn read_stream(&'a self, stream_to_read: String, sz: usize) -> CallResult {
        // REVIEW: Does the length of the empty vector matter here? If so, it seems like there is a better
        // way to do this, as this input vec is not used in constructing the output
        let input = vec![0; sz];
        BitcodeContext::log(&format!("input len = {}", input.len()));
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
    ///
    /// [Example](https://github.com/eluv-io/elv-wasm/blob/019b88ac27635d5022c2211751f6af5957df2463/samples/external/src/lib.rs#L133)
    ///
    /// REVIEW: I'm still somewhat confused on what this does after reading the example.
    /// What is the "output stream"? Is that documented somewhere? Does this just tell the fabric
    /// that I wrote something somewhere?
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

    // REVIEW: Maybe name this `make_success_string` to differentiate from `make_success_json`?
    pub fn make_success(&'a self, msg: &str) -> CallResult {
        let js_ret = json!({"jpc":"1.0", "id": self.request.id, "result" : msg});
        // REVIEW: Why are you converting string -> vec<u8> -> string? Why not just output msg?
        let v = serde_json::to_vec(&js_ret)?;
        let out = std::str::from_utf8(&v)?;
        elv_console_log(&format!("returning : {out}"));
        Ok(v)
    }

    // REVIEW: Why does this one have id as an argument? If it is an associated method with this
    // bitcode context, it should only ever use this bitcode context's id.
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

    // REVIEW: Similar comments as above. Also, these three functions can definitely be
    // consolidated. They only differ in message construction
    pub fn make_success_bytes(&'a self, msg: &[u8], id: &str) -> CallResult {
        let res: serde_json::Value = serde_json::from_slice(msg)?;
        let js_ret = json!({"jpc":"1.0", "id": id, "result" : res});
        let v = serde_json::to_vec(&js_ret)?;
        Ok(v)
    }

    pub fn make_error(&'a self, msg: &'static str) -> CallResult {
        make_json_error(ErrorKinds::Invalid(msg.to_string()), &self.request.id)
    }

    pub fn make_error_with_kind(&'a self, kind: ErrorKinds) -> CallResult {
        make_json_error(kind, &self.request.id)
    }

    // REVIEW: This is unused, and also doesn't make use of the error
    pub fn make_error_with_error<T>(&'a self, kind: ErrorKinds, _err: T) -> CallResult {
        make_json_error(kind, &self.request.id)
    }

    // REVIEW: What is this for? It's unused, and the arguments are not super helpful. Seems like an outdated version of call_function?
    pub fn call(&self, ns: &str, op: &str, msg: &[u8]) -> CallResult {
        host_call(self.request.id.as_str(), ns, op, msg)
    }

    /// call_function - enables the calling of fabric api's
    /// # Arguments
    /// * `fn_name` - the fabric api to call e.g. QCreateFileFromStream
    /// * `params` - a json block to pass as parameters to the function being called
    /// REVIEW: What is the link below supposed to be to?
    /// * `module` - one of {"core", "ctx", "ext"} see [fabric API]
    ///
    ///  This is the main workhorse function for the invoking of fabric bitcode APIs
    ///  wherein all the outer wrapper functions merely call this with the appropriate json parameters
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
        // REVIEW: IMO rust's debug formatting is better suited to this than using serde_json again,
        // but this is more of a personal preference thing
        elv_console_log(&format!("CALL STRING = {:?}", response));
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
                        // REVIEW: Should this really return ok in that case? When would this
                        // happen, to have neither a result or error
                        return Ok(call_ret_val);
                    }
                };
            }
        };
    }

    /// call_external_bitcode - enables the calling of fabric api's
    /// # Arguments
    /// * `function` - the function to call on the external bitcode
    /// * `args` -  the argumaents to pass the external function
    /// * `object_hash`  - the content object containing the external bitcode part
    /// * `code_part_hash` - the code part for the external bitcode
    ///
    ///   [Example](https://github.com/eluv-io/elv-wasm/blob/019b88ac27635d5022c2211751f6af5957df2463/samples/external/src/lib.rs#L101)
    ///
    /// ```
    /// use elvwasm::ErrorKinds;
    /// extern crate wapc_guest as guest;
    /// use serde_json::{json, Value};
    /// use std::str;
    /// use guest::CallResult;
    /// use elvwasm::ExternalCallResult;
    ///
    /// fn do_something<'s>(bcc: &'s elvwasm::BitcodeContext) -> wapc_guest::CallResult {
    ///     let http_p = &bcc.request.params.http;
    ///     let qp = &http_p.query;
    ///     let id = &bcc.request.id;
    ///     let img_hash = &qp.get("img_hash").ok_or(ErrorKinds::Invalid("img_hash not present".to_string()))?[0];
    ///     let img_obj= &qp.get("img_obj").ok_or(ErrorKinds::Invalid("img_obj not present".to_string()))?[0];
    ///     let tar_hash = &qp.get("tar_hash").ok_or(ErrorKinds::Invalid("tar_hash not present".to_string()))?[0];
    ///     bcc.log_info(&format!("img_hash ={img_hash:?} tar_hash = {tar_hash:?}"))?;
    ///     let params = json!({
    ///         "http" : {
    ///             "verb" : "some",
    ///             "headers": {
    ///                 "Content-type": [
    ///                     "application/json"
    ///                 ]
    ///             },
    ///             "path" : "/image/default/assets/birds.jpg",
    ///             "query" : {
    ///                 "height" : ["200"],
    ///             },
    ///         },
    ///     });
    ///     let img = bcc.call_external_bitcode("image", &params, img_obj, img_hash)?;
    ///     let exr:ExternalCallResult = serde_json::from_slice(&img)?;
    ///     let imgbits = base64::decode(&exr.fout)?;
    ///     Ok(imgbits)
    /// }
    /// ```

    pub fn call_external_bitcode(
        &'a self,
        function: &str,
        args: &serde_json::Value,
        object_hash: &str,
        code_part_hash: &str,
    ) -> CallResult {
        let params = json!({ "function": function,  "params" : args, "object_hash" : object_hash, "code_part_hash" : code_part_hash});
        let call_val = serde_json::to_vec(&params)?;
        let call_str = serde_json::to_string(&params)?;

        elv_console_log(&format!("CALL STRING = {call_str}"));
        let call_ret_val = host_call(
            self.request.id.as_str(),
            "ctx",
            "CallExternalBitcode",
            &call_val,
        )?;
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
    /// - stream:    the stream id (see [`new_file_stream`](Self::new_file_stream) or [`new_stream`](Self::new_stream))
    /// # Returns
    /// the checksum as hex-encoded string
    ///
    /// [Example](https://github.com/eluv-io/elv-wasm/blob/019b88ac27635d5022c2211751f6af5957df2463/samples/external/src/lib.rs#L109)
    ///
    pub fn close_stream(&'a self, stream: String) -> CallResult {
        self.call_function("CloseStream", json!({ "stream_id": stream }), "ctx")
    }

    // REVIEW: What does it mean for this to be a 'fabric bitcode stream'? Is this different than in
    // other places where it is called a 'fabric stream'? What are the stream semantics (does writing
    // notify the receiver, is it bidirectional, etc). These things should be visible from this doc (and the one below)
    // Also, it might be nice to have this function actually handle the call result for the user. So
    // the (simplified) signature and docstring would look more like
    //
    // Create a new stream, and either return the stream_id upon success or an error
    // pub fn new_stream(&'a self) -> Result<String, ErrorType> {}
    /// new_stream creates a new fabric bitcode stream.
    /// # Returns
    /// * output \[u8\] of format `{"stream_id" : id}` where id is a string
    pub fn new_stream(&'a self) -> CallResult {
        let v = json!({});
        self.call_function("NewStream", v, "ctx")
    }

    /// new_file_stream creates a new fabric file
    /// # Returns
    /// utf8-encoded JSON bytes formatted like the below. id and path are strings.
    /// ```json
    /// {
    ///   "stream_id": id,
    ///   "file_name": path,
    /// }
    /// ```
    pub fn new_file_stream(&'a self) -> CallResult {
        let v = json!({});
        self.call_function("NewFileStream", v, "ctx")
    }

    /// q_download_file : downloads the file stored at the fabric file location path for some content
    /// # Arguments
    /// *  `path` : fabric file location in the content
    /// *  `hash_or_token` : hash for the content containing the file
    ///
    pub fn q_download_file(&'a mut self, path: &str, hash_or_token: &str) -> CallResult {
        elv_console_log(&format!(
            "q_download_file path={path} token={hash_or_token}"
        ));
        let strm_bytes = self.new_stream()?;
        let strm_json: serde_json::Value = serde_json::from_slice(&strm_bytes)?["stream_id"];
        // REVIEW: This didn't correctly handle a failure of stream id parsing. Indexing into the
        // map will return serde_json::value::Value::Null if it doesn't exist, which serializes to
        // "null", which isn't empty
        // Actually, after reading further this should maybe use a struct similar to FileStream?
        // REVIEW: Should this close the stream?
        if strm_json.is_null() {
            return self
                .make_error_with_kind(ErrorKinds::IO(format!("Unable to create new stream")));
        }
        let sid = strm_json.to_string();
        let j = json!({
          "stream_id" : sid,
          "path" : path,
          "hash_or_token": hash_or_token,
        });

        let v: serde_json::Value = match self.call_function("QFileToStream", j, "core") {
            Err(e) => {
                return self.make_error_with_kind(ErrorKinds::NotExist(format!(
                    "QFileToStream failed path={path}, hot={hash_or_token} sid={sid} e={e}"
                )))
            }
            // REVIEW: Why use `unwrap_or_default` here? It seems to make more sense to error if we
            // cannot parse the file
            Ok(e) => serde_json::from_slice(&e).unwrap_or_default(),
        };

        elv_console_log(&format!("json={}", v.to_string()));
        let written_opt = v["written"].as_u64();

        match written_opt {
            Some(written) => self.read_stream(sid, written as usize),
            None => self.make_error_with_kind(ErrorKinds::NotExist(format!(
                "wrote 0 bytes, sid={sid} path={path}, hot={hash_or_token}"
            ))),
        }
    }

    /// q_upload_file : uploads the input data and stores it at the fabric file location as filetype mime
    /// # Arguments
    /// *  `qwt` : a fabric write token
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
        // REVIEW: Should this really just create a file with size 0 if we fail to parse? Seems like
        // it should just fail instead
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

    // REVIEW: Seems a bit 'rustier' to make this return Option<usize>, so that we can differentiate
    // the errors or other things. Or maybe even a Result<Option<usize>, ErrorOfSomeType> so we can
    // distinguish transport/parsing errors from streams that are not open from open streams with 0 bytes
    /// file_stream_size computes the current size of a fabric file stream given its stream name
    ///     filename : the name of the file steam.  See [`new_file_stream`](Self::new_file_stream).
    pub fn file_stream_size(&'a self, filename: &str) -> usize {
        elv_console_log("file_stream_size");
        let ret: Vec<u8> =
            match self.call_function("FileStreamSize", json!({ "file_name": filename }), "ctx") {
                Ok(m) => m,
                Err(_e) => {
                    return 0;
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

#[test]
fn null_json_isnt_empty() {
    println!("{}", Value::Null.to_string());
    println!("{}", Value::Null.to_string().is_empty());
}

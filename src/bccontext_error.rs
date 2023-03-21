extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate thiserror;
extern crate wapc_guest as guest;

use crate::elv_console_log;
use serde_derive::Serialize;
use serde_json::json;
use thiserror::Error;
use wapc_guest::CallResult;

#[derive(Error, Debug, Clone, Serialize)]
#[repr(u8)]
pub enum ErrorKinds {
    #[error("Other Error : {0}")]
    Other(String),
    #[error("NotImplemented : {0}")]
    NotImplemented(String),
    #[error("Invalid : {0}")]
    Invalid(String),
    #[error("Permission : {0}")]
    Permission(String),
    #[error("IO : {0}")]
    IO(String),
    #[error("Exist : {0}")]
    Exist(String),
    #[error("NotExist : {0}")]
    NotExist(String),
    #[error("IsDir : {0}")]
    IsDir(String),
    #[error("NotDir : {0}")]
    NotDir(String),
    #[error("Finalized : {0}")]
    Finalized(String),
    #[error("NotFinalized : {0}")]
    NotFinalized(String),
    #[error("BadHttpParams : {0}")]
    BadHttpParams(String),
}

fn discriminant(v: &ErrorKinds) -> u8 {
    unsafe { *(v as *const ErrorKinds as *const u8) }
}
/// make_json_error translates the bitcode [ErrorKinds] to an error response to the client
/// # Arguments
/// * `err`- the error to be translated to a response
pub fn make_json_error(err: ErrorKinds, id: &str) -> CallResult {
    let msg = json!(
      {
        "error" :  {
          "op" : discriminant(&err),
          "desc" : err,
          "data" : {
            "op" : discriminant(&err),
            "desc" : err,
          },
        },
        "jpc" : "1.0",
        "id"  : id,
      }
    );
    let vr = serde_json::to_vec(&msg)?;
    // let out = std::str::from_utf8(&vr)?;
    // elv_console_log(&format!("returning a test {out}"));
    Ok(vr)
}

pub fn make_success_json(msg: &serde_json::Value, id: &str) -> CallResult {
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

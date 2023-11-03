extern crate chrono;
extern crate elvwasm;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate serde_with;

use chrono::offset::Utc;
use chrono::DateTime;
use elvwasm::{
    implement_bitcode_module, jpc, make_json_error, make_success_json, register_handler, ErrorKinds,
};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use serde_with::serde_as;
use std::collections::HashMap;

implement_bitcode_module!("playout_selector", do_playout_selector);

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Type {
    pub code: u8,
    pub format: u8,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Hash {
    #[serde(rename = "type")]
    pub q_type: Type,
    pub digest: Vec<u8>,
    pub size: i64,
    pub preamble_size: i64,
    pub id: Vec<u8>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub expiration: DateTime<Utc>,
    pub s: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AutoUpdate {
    #[serde(default)]
    pub tag: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Extra {
    #[serde(default)]
    pub auto_update: AutoUpdate,
    #[serde(default)]
    pub container: String,
    #[serde(default)]
    pub resolution_error: String,
    #[serde(default)]
    pub authorization: String,
    #[serde(default)]
    pub enforce_auth: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Link {
    #[serde(default)]
    pub target: Hash,
    #[serde(default)]
    pub selector: String,
    #[serde(default)]
    pub path: Vec<String>,
    #[serde(default)]
    pub off: i64,
    #[serde(default)]
    pub len: i64,
    #[serde(default)]
    pub props: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub extra: Extra,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SelectorItem {
    #[serde(default)]
    pub playout: Link,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SelectorGroup {
    members: Vec<String>,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayoutSelector {
    groups: Vec<SelectorGroup>,
    items: Vec<SelectorItem>,
}

/// .
///
/// # Errors
///
/// This function will return an error if .
#[no_mangle]
fn do_playout_selector(bcc: &mut elvwasm::BitcodeContext) -> CallResult {
    let id = &bcc.request.id;
    let http_p = &bcc.request.params.http;
    if http_p.verb != "GET" {
        return make_json_error(
            ErrorKinds::Invalid("playout selector must be called with GET".to_string()),
            id,
        );
    }
    let meta: PlayoutSelector = serde_json::from_slice(&bcc.sqmd_get_json("playout_selector")?)?;

    let _qp = &http_p.query;
    make_success_json(
        &json!(
        {
            "headers" : "application/json",
            "body" : "SUCCESS",
            "result" : "complete",
        }),
        id,
    )
}

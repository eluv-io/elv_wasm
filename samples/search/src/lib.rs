#![feature(generic_associated_types)]

mod old_man;

pub mod crawler;
pub mod graph;
pub mod indexer;
pub mod searcher;
pub mod utils;

extern crate elvwasm;
extern crate serde_json;

use serde_json::{json, Value, Map};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::old_man::S_OLD_MAN;
use elvwasm::ErrorKinds;
use indexer::Indexer;

use elvwasm::{implement_bitcode_module, jpc, register_handler, BitcodeContext};

implement_bitcode_module!("crawl", do_crawl, "more_crawl", do_crawl2, "even_more_crawl", do_crawl3, "search_update", do_search_update, "search_update_new", do_search_update_new);

fn extract_body(v:Value) -> Option<Value>{
    let obj = match v.as_object(){
        Some(v) => v,
        None => return None,
    };
    let mut full_result = true;
    let res = match obj.get("result"){
        Some(m) => m,
        None => match obj.get("http"){
            Some(h) => {
                full_result = false;
                h
            },
            None => return None,
        },
    };
    if full_result{
        let http = match res.get("http"){
            Some(h) => h,
            None => return None
        };
        return http.get("body").cloned();
    }
    return res.get("body").cloned();
}

fn do_crawl3<>(bcc: &mut elvwasm::BitcodeContext<>) -> CallResult {
    do_crawl2(bcc)
}

fn do_crawl2<>(bcc: &mut elvwasm::BitcodeContext<>) -> CallResult {
    do_crawl(bcc)
}

fn do_crawl<>(bcc: &mut elvwasm::BitcodeContext<>) -> CallResult {
    let http_p = &bcc.request.params.http;
    let qp = &http_p.query;
    BitcodeContext::log(&format!("In do_crawl hash={} headers={:#?} query params={:#?}",&bcc.request.q_info.hash, &http_p.headers, qp));
    let id = &bcc.request.id;
    let mut v = json!({});
    BitcodeContext::log("before BUILDER");
    bcc.new_index_builder(v)?;
    BitcodeContext::log("NEW INDEX BUILDER");
    v = json!({ "field_name": "title", "type": 1_u8, "stored": true});
    let field_title_vec = bcc.builder_add_text_field(v)?;
    let ft_json:serde_json::Value = serde_json::from_slice(&field_title_vec)?;
    let field_title = match extract_body(ft_json){
        Some(o) => o.get("field").unwrap().as_u64(),
        None => return bcc.make_error_with_kind(ErrorKinds::BadHttpParams("could not find key document-create-id")),
    };
    BitcodeContext::log("TEXT FILED 1");
    v = json!({ "field_name": "body", "type": 2_u8 , "stored": false});
    let field_body_vec = bcc.builder_add_text_field(v)?;
    let fb_json:serde_json::Value = serde_json::from_slice(&field_body_vec)?;
    let field_body = match extract_body(fb_json){
        Some(o) => o.get("field").unwrap().as_u64(),
        None => return bcc.make_error_with_kind(ErrorKinds::BadHttpParams("could not find key document-create-id")),
    };
    BitcodeContext::log("TEXT FILED 2");
    v = json!({});
    bcc.builder_build(v.clone())?;
    let doc_old_man_u8 = bcc.document_create(v)?;
    BitcodeContext::log("DOC CREATE");
    let doc_old_man:serde_json::Value = serde_json::from_slice(&doc_old_man_u8)?;
    console_log(&format!("obj_old = {:?}", &doc_old_man));
    let doc_id = match extract_body(doc_old_man){
        Some(o) => o.get("document-create-id").unwrap().as_u64(),
        None => return bcc.make_error_with_kind(ErrorKinds::BadHttpParams("could not find key document-create-id")),
    };
    v = json!({ "field": field_title, "value": "The Old Man and the Sea", "doc_id": doc_id});
    bcc.document_add_text(v)?;
    BitcodeContext::log("DOC ADD TEXT TITLE");
    v = json!({ "field": field_body, "value": S_OLD_MAN, "doc_id": doc_id});
    bcc.document_add_text(v)?;
    BitcodeContext::log("DOC ADD TEXT BODY");
    v = json!({});
    bcc.document_create_index(v.clone())?;
    bcc.index_create_writer(v)?;
    v = json!({ "document_id": doc_id});
    bcc.index_add_document(v)?;
    v = json!({});
    bcc.index_writer_commit(v)?;
    let part_u8 = bcc.archive_index_to_part()?;
    let part_hash:serde_json::Value = serde_json::from_slice(&part_u8)?;
    let b = extract_body(part_hash.clone());
    let body_hash = b.unwrap_or_else(|| json!({}));
    BitcodeContext::log(&format!("part hash = {}, bosy = {}", &part_hash.to_string(), &body_hash.to_string()));
    bcc.make_success_json(&json!(
        {
            "headers" : "application/json",
            "body" : "SUCCESS",
            "result" : body_hash,
        }), id)
}

#[derive(Serialize, Deserialize)]
pub struct FieldDefinitions {
    #[serde(default)]
    pub field_type: String,
    #[serde(default)]
    pub options: Map<String, Value>,
    #[serde(default)]
    pub paths: Vec<String>,
}

fn merge(a: &mut Value, b: Value) {
    if let Value::Object(a) = a {
        if let Value::Object(b) = b {
            for (k, v) in b {
                if v.is_null() {
                    a.remove(&k);
                }
                else {
                    merge(a.entry(k).or_insert(Value::Null), v);
                }
            }

            return;
        }
    }

    *a = b;
}

fn do_search_update_new<>(bcc: &mut elvwasm::BitcodeContext<>) -> CallResult {
    let res = bcc.sqmd_get_json("/indexer/arguments/fields")?;
    let fields:Map<String, Value> = serde_json::from_slice(&res)?;
    let mut idx_fields = HashMap::<String, indexer::FieldConfig>::new();
    for (field, val) in fields.into_iter(){
        idx_fields.insert(field, serde_json::from_value(val).unwrap());
    }
    let _idx = Indexer::new(bcc, "idx".to_string(), idx_fields)?;
    let id = &bcc.request.id;

    let res_config = bcc.sqmd_get_json("/indexer/config")?;
    let fields_config: Value = serde_json::from_slice(&res_config)?;
    let _indexer_config = crawler::IndexerConfig::parse_index_config(&fields_config)?;


    bcc.make_success_json(&json!(
        {
            "headers" : "application/json",
            "body" : "SUCCESS",
            "result" : {"status" : "update complete"},
        }), id)
}

fn do_search_update<>(bcc: &mut elvwasm::BitcodeContext<>) -> CallResult {
    let http_p = &bcc.request.params.http;
    let _qp = &http_p.query;
    let id = &bcc.request.id;
    bcc.new_index_builder(json!({}))?;
    let mut extra_fields = json!({});
    let res = bcc.sqmd_get_json("/indexer/arguments/fields")?;
    let fields:Map<String, Value> = serde_json::from_slice(&res)?;
    for (field, val) in fields.into_iter(){
        let new_field =json!({format!("f_{}", field):&val});
        merge(&mut extra_fields, new_field);
    }
    let v = json!({});
    bcc.builder_build(v.clone())?;
    let mut core_fields = json!({});
    let core_field_names = vec!["id", "hash", "type", "qlib_id", "has_field", "prefix", "display_title", "asset_type", "title_type"];
    // create core fields schema
    for key in core_field_names{
        core_fields[key] = json!({"options" :  {"builder": {}, "stats": {"simple": false, "histogram": false}}, "type":"string"})
    }

    let mut all_fields = json!({});
    merge(&mut all_fields, core_fields);
    merge(&mut all_fields, extra_fields);

    let res = bcc.sqmd_get_json("/indexer/arguments/document/prefix")?;
    let _document_prefix_filter:String = serde_json::from_slice(&res)?;

    // core_fields = {key: {
    //     "options": {"builder": {}, "stats": {"simple": False, "histogram": False}},
    //     "type": "string",
    // } for key in core_field_names}

        //let fd:FieldDefinitions = serde_json::from_value(val.clone())?;
    //let v = json!({ "field_name": field, "type": fd.field_type, "stored": true});
    //bcc.builder_add_text_field(v)?;

    bcc.make_success_json(&json!(
        {
            "headers" : "application/json",
            "body" : "SUCCESS",
            "result" : {"status" : "update complete"},
        }), id)

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

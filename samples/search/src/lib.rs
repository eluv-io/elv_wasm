#![feature(generic_associated_types)]

mod old_man;

pub mod crawler;
pub mod graph;
pub mod indexer;
pub mod searcher;
pub mod utils;

extern crate elvwasm;
extern crate serde_json;

use crawler::FieldConfig;
use serde_json::{json, Value, Map};
use serde::{Deserialize, Serialize};
use crate::old_man::S_OLD_MAN;
use elvwasm::ErrorKinds;
use indexer::Indexer;

use elvwasm::{implement_bitcode_module, jpc, register_handler, BitcodeContext};

implement_bitcode_module!( "search", do_search, "crawl", do_crawl, "more_crawl", do_crawl2, "even_more_crawl", do_crawl3, "search_update", do_search_update, "search_update_new", do_search_update_new);

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
    let td = bcc.temp_dir()?;
    let dir:&str = serde_json::from_slice(&td)?;
    BitcodeContext::log("before BUILDER");
    bcc.new_index_builder(json!({"directory" : dir}))?;
    BitcodeContext::log("NEW INDEX BUILDER");
    let field_title_vec = bcc.builder_add_text_field(json!({ "field_name": "title", "type": 2_u8, "stored": true}))?;
    let ft_json:serde_json::Value = serde_json::from_slice(&field_title_vec)?;
    let field_title = match extract_body(ft_json){
        Some(o) => o.get("field").unwrap().as_u64(),
        None => return bcc.make_error_with_kind(ErrorKinds::BadHttpParams("could not find key document-create-id")),
    };
    BitcodeContext::log("TEXT FILED 1");
    let field_body_vec = bcc.builder_add_text_field(json!({ "field_name": "body", "type": 2_u8 , "stored": true}))?;
    let fb_json:serde_json::Value = serde_json::from_slice(&field_body_vec)?;
    let field_body = match extract_body(fb_json){
        Some(o) => o.get("field").unwrap().as_u64(),
        None => return bcc.make_error_with_kind(ErrorKinds::BadHttpParams("could not find key document-create-id")),
    };
    BitcodeContext::log("TEXT FILED 2");
    bcc.builder_build(json!({}))?;
    let doc_old_man_u8 = bcc.document_create(json!({}))?;
    BitcodeContext::log("DOC CREATE");
    let doc_old_man:serde_json::Value = serde_json::from_slice(&doc_old_man_u8)?;
    console_log(&format!("obj_old = {:?}", &doc_old_man));
    let o_doc_id = match extract_body(doc_old_man){
        Some(o) => o.get("document-create-id").unwrap().as_u64(),
        None => return bcc.make_error_with_kind(ErrorKinds::BadHttpParams("could not find key document-create-id")),
    };
    let doc_id = o_doc_id.unwrap();
    bcc.log_info(&format!("doc_id={}, field_title = {}, field_body={}", doc_id, field_title.unwrap(), field_body.unwrap()))?;
    bcc.document_add_text(json!({ "field": field_title.unwrap(), "value": "The Old Man and the Sea", "doc_id": doc_id}))?;
    BitcodeContext::log("DOC ADD TEXT TITLE");
    bcc.document_add_text(json!({ "field": field_body.unwrap(), "value": S_OLD_MAN, "doc_id": doc_id}))?;
    BitcodeContext::log("DOC ADD TEXT BODY");
    bcc.document_create_index(json!({}))?;
    bcc.index_create_writer(json!({}))?;
    bcc.index_add_document(json!({ "document_id": doc_id}))?;
    bcc.index_writer_commit(json!({}))?;
    let part_u8 = bcc.archive_index_to_part(dir)?;
    let part_hash:serde_json::Value = serde_json::from_slice(&part_u8)?;
    let b = extract_body(part_hash.clone());
    let body_hash = b.unwrap_or_else(|| json!({}));
    bcc.callback(200, "application/json", part_u8.len())?;
    BitcodeContext::write_stream_auto(id.clone(), "fos", &part_u8)?;
    BitcodeContext::log(&format!("part hash = {}, bosy = {}", &part_hash.to_string(), &body_hash.to_string()));
    bcc.make_success_json(&json!(
        {
            "headers" : "application/json",
            "body" : "SUCCESS",
            "result" : 0,
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
// fn do_search<>(bcc: &mut elvwasm::BitcodeContext<>) -> CallResult {
//     bcc.log_info("In do search")?;
//     let id = &bcc.request.id;
//     bcc.make_success_json(&json!(
//         {
//             "headers" : "application/json",
//             "body" : "SUCCESS",
//             "result" : 0,
//         }), id)
// }
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Restore {
    http: HttpP,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct HttpP {
    body: String,
}


fn do_search<>(bcc: &mut elvwasm::BitcodeContext<>) -> CallResult {
    bcc.log_info("In do search")?;
    let id = &bcc.request.id;
    let http_p = &bcc.request.params.http;
    bcc.log_info(&format!("http={:?}", &http_p))?;
    let qp = &http_p.query;
    if qp.len() == 0{
        bcc.log_info("qp len 0")?;
        return bcc.make_error("query params are empty");
    }
    bcc.log_info("here")?;

    let part_hash = &qp["part-hash"][0];
    let content_hash = &qp["content-hash"][0];


    let index_bytes = bcc.restore_index_from_part(content_hash, part_hash)?;
    let index_dir:Restore = serde_json::from_slice(&index_bytes)?;
    bcc.log_info(&format!("directory={}", index_dir.http.body))?;
    bcc.new_index_builder(json!({"directory" : index_dir.http.body}))?;
    let field_title_vec = bcc.builder_add_text_field(json!({ "field_name": "title", "type": 2_u8, "stored": true}))?;
    let ft_json:serde_json::Value = serde_json::from_slice(&field_title_vec)?;
    let _field_title = match extract_body(ft_json){
        Some(o) => o.get("field").unwrap().as_u64(),
        None => return bcc.make_error_with_kind(ErrorKinds::BadHttpParams("could not find key document-create-id")),
    };
    BitcodeContext::log("TEXT FILED 1");
    let field_body_vec = bcc.builder_add_text_field(json!({ "field_name": "body", "type": 2_u8 , "stored": true}))?;
    let fb_json:serde_json::Value = serde_json::from_slice(&field_body_vec)?;
    let _field_body = match extract_body(fb_json){
        Some(o) => o.get("field").unwrap().as_u64(),
        None => return bcc.make_error_with_kind(ErrorKinds::BadHttpParams("could not find key document-create-id")),
    };
    BitcodeContext::log("TEXT FILED 2");
    bcc.builder_build(json!({}))?;

    bcc.builder_create_index(json!({}))?;
    bcc.index_reader_builder_create(json!({}))?;
    bcc.index_reader_searcher(json!({}))?;
    let v_index_parser = json!({"fields" : ["title", "body"]});
    bcc.query_parser_for_index(v_index_parser)?;
    bcc.query_parser_parse_query("Sea")?;
    // let v_index_parser = serde_json::from_str(r#"{
    //     "fields" : ["title", "body"]
    //   }
    // }"#).unwrap();
    let res = bcc.query_parser_search(json!({}))?;
    bcc.callback(200, "application/json", res.len())?;
    BitcodeContext::write_stream_auto(id.clone(), "fos", &res)?;
    bcc.make_success_json(&json!(
        {
            "headers" : "application/json",
            "body" : "SUCCESS",
            "result" : 0,
        }), id)
}

fn do_search_update_new<>(bcc: &mut elvwasm::BitcodeContext<>) -> CallResult {
    let res = bcc.sqmd_get_json("/indexer/arguments/fields")?;
    let fields:Map<String, Value> = serde_json::from_slice(&res)?;
    let mut idx_fields = Vec::<crawler::FieldConfig>::new();
    for (field, val) in fields.into_iter(){
        let fc_cur = FieldConfig{
            name: field,
            options: serde_json::from_value(val["options"].clone()).unwrap(),
            field_type: serde_json::from_value(val["field_type"].clone()).unwrap(),
            paths: serde_json::from_value(val["paths"].clone()).unwrap()
        };

        idx_fields.push(fc_cur);
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
    let td = bcc.temp_dir()?;
    let dir:&str = serde_json::from_slice(&td)?;
    bcc.new_index_builder(json!({"directory": dir}))?;
    let mut extra_fields = json!({});
    let res = bcc.sqmd_get_json("/indexer/arguments/fields")?;
    let fields:Map<String, Value> = serde_json::from_slice(&res)?;
    for (field, val) in fields.into_iter(){
        let new_field =json!({format!("f_{}", field):&val});
        merge(&mut extra_fields, new_field);
    }
    let v = json!({});
    bcc.builder_build(v)?;
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

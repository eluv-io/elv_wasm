mod old_man;

extern crate elvwasm;
extern crate serde_json;
use serde_json::{json, Value};
use crate::old_man::S_OLD_MAN;
use elvwasm::ErrorKinds;

use elvwasm::{implement_bitcode_module, jpc, register_handler, BitcodeContext};

implement_bitcode_module!("crawl", do_crawl, "more_crawl", do_crawl2, "even_more_crawl", do_crawl3);

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
        return match http.get("body"){
            Some(b) => Some(b.clone()),
            None => None
        };
    }
    return match res.get("body"){
        Some(b) => Some(b.clone()),
        None => None
    };
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
    BitcodeContext::log(&format!("before BUILDER"));
    bcc.new_index_builder(v)?;
    BitcodeContext::log(&format!("NEW INDEX BUILDER"));
    v = json!({ "field_name": "title", "type": 1 as u8, "stored": true});
    let field_title_vec = bcc.builder_add_text_field(v)?;
    let ft_json:serde_json::Value = serde_json::from_slice(&field_title_vec)?;
    let field_title = match extract_body(ft_json.clone()){
        Some(o) => o.get("field").unwrap().as_u64(),
        None => return bcc.make_error_with_kind(ErrorKinds::BadHttpParams("could not find key document-create-id")),
    };
    BitcodeContext::log(&format!("TEXT FILED 1"));
    v = json!({ "field_name": "body", "type": 2 as u8 , "stored": false});
    let field_body_vec = bcc.builder_add_text_field(v)?;
    let fb_json:serde_json::Value = serde_json::from_slice(&field_body_vec)?;
    let field_body = match extract_body(fb_json.clone()){
        Some(o) => o.get("field").unwrap().as_u64(),
        None => return bcc.make_error_with_kind(ErrorKinds::BadHttpParams("could not find key document-create-id")),
    };
    BitcodeContext::log(&format!("TEXT FILED 2"));
    v = json!({});
    bcc.builder_build(v.clone())?;
    let doc_old_man_u8 = bcc.document_create(v)?;
    BitcodeContext::log(&format!("DOC CREATE"));
    let doc_old_man:serde_json::Value = serde_json::from_slice(&doc_old_man_u8)?;
    console_log(&format!("obj_old = {:?}", &doc_old_man));
    let doc_id = match extract_body(doc_old_man.clone()){
        Some(o) => o.get("document-create-id").unwrap().as_u64(),
        None => return bcc.make_error_with_kind(ErrorKinds::BadHttpParams("could not find key document-create-id")),
    };
    v = json!({ "field": field_title, "value": "The Old Man and the Sea", "doc_id": doc_id});
    bcc.document_add_text(v)?;
    BitcodeContext::log(&format!("DOC ADD TEXT TITLE"));
    v = json!({ "field": field_body, "value": S_OLD_MAN, "doc_id": doc_id});
    bcc.document_add_text(v)?;
    BitcodeContext::log(&format!("DOC ADD TEXT BODY"));
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
    let body_hash = b.unwrap_or(json!({}));
    BitcodeContext::log(&format!("part hash = {}, bosy = {}", &part_hash.to_string(), &body_hash.to_string()));
    bcc.make_success_json(&json!(
        {
            "headers" : "application/json",
            "body" : "SUCCESS",
            "result" : body_hash,
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

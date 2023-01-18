extern crate elvwasm;
extern crate serde_json;
extern crate serde;
extern crate base64;
#[macro_use(defer)] extern crate scopeguard;

use elvwasm::{implement_bitcode_module, jpc, register_handler, ErrorKinds, ExternalCallResult, NewStreamResult, CreatePartResult, FinalizeCallResult};
use serde_json::{json};
use elvwasm::BitcodeContext;

implement_bitcode_module!("external", do_external);

/*
        "http" : {
            "verb" : "some",
            "path" : "/image/default/assets/birds.jpg",
            "query" : {
                "height" : 200,
            },
        }
*/


#[no_mangle]
fn do_external(bcc: &mut BitcodeContext) -> CallResult {
    let http_p = &bcc.request.params.http;
    let qp = &http_p.query;
    let id = &bcc.request.id;
    let img_hash = &qp.get("img_hash").ok_or(ErrorKinds::Invalid("img_hash not present".to_string()))?[0];
    let img_obj= &qp.get("img_obj").ok_or(ErrorKinds::Invalid("img_hash not present".to_string()))?[0];
    let tar_hash = &qp.get("tar_hash").ok_or(ErrorKinds::Invalid("tar_hash not present".to_string()))?[0];
    bcc.log_info(&format!("img_hash ={img_hash:?} tar_hash = {tar_hash:?}"))?;
    let params = json!({
        "jpc" : "1.0",
        "id" : id,
        "method" : "content",
        "params" :{
            "http" : {
                "verb" : "some",
                "headers": {
                    "Content-type": [
                      "application/json"
                    ]
                },
                "path" : "/image/default/assets/birds.jpg",
                "query" : {
                    "height" : ["200"],
                },
            },
        },
        "qinfo" : bcc.request.q_info.clone(),
    });
    let img = bcc.call_external_bitcode("image", &params, img_obj, img_hash)?;
    let exr:ExternalCallResult = serde_json::from_slice(&img)?;
    let imgbits = &base64::decode(&exr.fout)?;
    BitcodeContext::log(&format!("imgbits decoded size = {} fout size = {}", imgbits.len(), exr.fout.len()));
    BitcodeContext::log(&format!("fout {}", &exr.fout));
    let stream_img:NewStreamResult = serde_json::from_slice(&bcc.new_stream()?)?;
    defer!{
        BitcodeContext::log(&format!("Closing part stream {}", &stream_img.stream_id));
        let _ = bcc.close_stream(stream_img.stream_id.clone());
    }
    bcc.write_stream(&stream_img.stream_id, imgbits, imgbits.len())?;
    let imgpart:CreatePartResult = serde_json::from_slice(&bcc.q_create_part_from_stream(&bcc.request.q_info.write_token, &stream_img.stream_id)?)?;
    BitcodeContext::log(&format!("imgpart hash {} size = {}", &imgpart.qphash, imgpart.size));
    let fc:FinalizeCallResult = serde_json::from_slice(&bcc.q_finalize_content(&bcc.request.q_info.write_token)?)?;
    let tar_params = json!({
        "jpc" : "1.0",
        "id" : id,
        "method" : "/tar",
        "params" :{
            "http" : {
                "verb" : "some",
                "headers": {
                    "Content-type": [
                      "application/json"
                    ]
                },
                "path" : "/tar",
                "query" : {
                    "object_id_or_hash" : [fc.qhash],
                },
            },
        },
        "qinfo" : bcc.request.q_info.clone(),
    });
    let exr_tar:ExternalCallResult = serde_json::from_slice(&bcc.call_external_bitcode("tar", &tar_params, &fc.qhash, tar_hash)?)?;
    let tarbits = &base64::decode(&exr_tar.fout)?;
    bcc.log_info(&format!("fout size = {} tar_ bit len = {}", &exr_tar.fout.len(), tarbits.len()))?;
    bcc.write_stream("fos", tarbits, tarbits.len())?;
    bcc.callback(200, "application/zip", tarbits.len())?;

    bcc.make_success_json(&json!(
        {
            "headers" : "application/json",
            "body" : "SUCCESS",
            "result" : "complete",
        }), id)
}
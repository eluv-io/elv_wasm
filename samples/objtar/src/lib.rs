#![feature(try_trait_v2)]
#![feature(linked_list_cursors)]
extern crate elvwasm;
extern crate serde_json;
extern crate serde;
extern  crate zip;
extern crate base64;
#[macro_use(defer)] extern crate scopeguard;

use std::convert::TryInto;

use elvwasm::{implement_bitcode_module, jpc, register_handler, QPartList, NewStreamResult, BitcodeContext, ReadResult};
use serde_json::{json};
use zip::write::{FileOptions};
use std::io::Write;
use std::str::from_utf8;
use base64::decode;

implement_bitcode_module!("tar", do_tar_from_obj);


#[no_mangle]
fn do_tar_from_obj(bcc: &mut elvwasm::BitcodeContext) -> CallResult {
    let http_p = &bcc.request.params.http;
    let qp = &http_p.query;
    let id = &bcc.request.id;
    let obj_id = match qp.get("object_id_or_hash"){
        Some(x) => x,
        None => return bcc.make_error("unable to locate query parameter object_id_or_hash"),
    };
    let plraw = bcc.q_part_list(obj_id[0].to_string())?;
    let s = match from_utf8(&plraw) {
        Ok(v) => v.to_string(),
        Err(_e) => return bcc.make_error_with_kind(elvwasm::ErrorKinds::Invalid("Part list not available err =")),
    };
    let pl:QPartList = serde_json::from_str(&s)?;

    let w = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(w);
    for part in pl.part_list.parts {
        let res = bcc.new_stream()?;
        let stream_wm: NewStreamResult = serde_json::from_slice(&res)?;
        defer!{
            BitcodeContext::log(&format!("Closing watermark stream {}", &stream_wm.stream_id));
            let _ = bcc.close_stream(stream_wm.stream_id.clone());
        }
        let _wprb = bcc.write_part_to_stream(stream_wm.stream_id.clone(), part.hash.clone(), bcc.request.q_info.hash.clone(), 0, -1)?;
        let usz = part.size.try_into()?;
        let data:ReadResult = serde_json::from_slice(&bcc.read_stream(stream_wm.stream_id.clone(), usz)?)?;
        zip.start_file(part.hash.clone(), FileOptions::default())?;
        BitcodeContext::log(&format!("zip starting {} part size = {usz} \n", part.hash.clone()));
        let b64_decoded = decode(&data.result)?;
        zip.write_all(&b64_decoded)?;
    }

    let zip_res = match zip.finish(){
        Ok(z) => z.into_inner(),
        Err(e) => return bcc.make_error_with_error(elvwasm::ErrorKinds::Invalid("zip failed to finish"), e),
    };

    bcc.callback(200, "application/zip", zip_res.len())?;
    BitcodeContext::write_stream_auto(bcc.request.id.clone(), "fos", &zip_res)?;


    bcc.make_success_json(&json!(
        {
            "headers" : "application/zip",
            "body" : "SUCCESS",
            "result" : 0,
        }), id)
}


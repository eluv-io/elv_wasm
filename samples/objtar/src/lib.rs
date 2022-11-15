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
    BitcodeContext::log("HERE");
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
        Err(e) => return bcc.make_error_with_kind(elvwasm::ErrorKinds::Invalid("Part list not available")),
    };
    BitcodeContext::log(&format!("return is {s}\n"));
    let pl:QPartList = serde_json::from_str(&s)?;
    BitcodeContext::log(&format!("pl {:?}\n", pl));

    const BUF_SIZE:usize = 65536;
    let buf: &mut [u8] = &mut [0u8; BUF_SIZE];
    let w = std::io::Cursor::new(buf);
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
        let mut steps = usz / BUF_SIZE;
        if steps == 0 && usz > 0{
            steps = 1
        }
        zip.start_file(part.hash.clone(), FileOptions::default())?;
        BitcodeContext::log(&format!("zip starting {} part size = {usz} \n", part.hash.clone()));
        for step in 0..steps{
            let mut write_size = BUF_SIZE;
            if step == steps-1{
                write_size = usz-1;
            }
            BitcodeContext::log(&format!("About to write start = {} end = {} len = {} data = {} ,result = {}\n", step, (step+1), write_size, data.ret.len(), data.result.len()));
            let b64_decoded = decode(&data.result)?;
            let w = zip.write(&b64_decoded)?;
            BitcodeContext::log(&format!("Wrote = {}\n", w));
        }
    }
    BitcodeContext::log("Done\n");

    let zip_res = zip.finish().unwrap().into_inner();

    bcc.callback(200, "application/zip", zip_res.len())?;
    BitcodeContext::write_stream_auto(bcc.request.id.clone(), "fos", &zip_res)?;


    bcc.make_success_json(&json!(
        {
            "headers" : "application/zip",
            "body" : "SUCCESS",
            "result" : 0,
        }), id)
}


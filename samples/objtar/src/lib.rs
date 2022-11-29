#![feature(try_trait_v2)]
#![feature(linked_list_cursors)]
extern crate elvwasm;
extern crate serde_json;
extern crate serde;
extern crate base64;
#[macro_use(defer)] extern crate scopeguard;

use std::convert::TryInto;

use elvwasm::{implement_bitcode_module, jpc, register_handler, QPartList, NewStreamResult, BitcodeContext, ReadResult};
use serde_json::{json};
use std::io::{Write, BufWriter, ErrorKind,SeekFrom};
use std::str::from_utf8;
use base64::decode;
use flate2::write::{GzEncoder};

implement_bitcode_module!("tar", do_tar_from_obj);
#[derive(Debug)]
struct FabricWriter<'a>{
    bcc:&'a BitcodeContext,
    size: *mut usize
}

impl<'a> FabricWriter<'a>{
    fn new(bcc:&'a BitcodeContext, sz:*mut usize) -> FabricWriter<'a>{
        FabricWriter{
            bcc :bcc,
            size:sz
        }
    }
}
impl<'a> std::io::Write for FabricWriter<'a>{
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error>{
        match BitcodeContext::write_stream_auto(self.bcc.request.id.clone(), "fos", buf){
            Ok(s) => {
                BitcodeContext::log(&format!("Wrote {} bytes", buf.len()));
                let w:elvwasm::WritePartResult = serde_json::from_slice(&s)?;
                unsafe{*(self.size) += w.written;}
                Ok(w.written)
            },
            Err(e) => Err(std::io::Error::new(ErrorKind::Other, e)),
        }
    }

    fn flush(&mut self) -> Result<(), std::io::Error>{
        Ok(())
    }
}

impl<'a> std::io::Seek for FabricWriter<'a>{
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, std::io::Error>{
        match pos{
            SeekFrom::Start(s) => {
                BitcodeContext::log(&format!("SEEK from START {s}"));
            },
            SeekFrom::Current(s) => {
                BitcodeContext::log(&format!("SEEK from CURRENT {s}"));

            },
            SeekFrom::End(s) => {
                BitcodeContext::log(&format!("SEEK from END {s}"));
            },
        }
        Ok(self.size as u64)
    }
}

#[no_mangle]
fn do_tar_from_obj(bcc: &mut elvwasm::BitcodeContext) -> CallResult {
    let http_p = &bcc.request.params.http;
    let qp = &http_p.query;
    let id = &bcc.request.id;
    let vqhot = &vec!(bcc.request.q_info.qhot());
    let obj_id = match qp.get("object_id_or_hash"){
        Some(x) => x,
        None => vqhot,
    };
    const DEF_CAP:usize = 1000000;
    let buf_cap = match qp.get("buffer_capacity"){
         Some(x) => {
            BitcodeContext::log(&format!("new capacity of {x:?} set"));
            x[0].parse().unwrap_or(DEF_CAP)
         },
        None => DEF_CAP,
    };
    let mut total_size = 0;
    let tsp = std::ptr::addr_of_mut!(total_size);
    let bw = BufWriter::with_capacity(buf_cap, FabricWriter::new(bcc, tsp));

    let plraw = bcc.q_part_list(obj_id[0].to_string())?;
    let s = match from_utf8(&plraw) {
        Ok(v) => v.to_string(),
        Err(e) => return bcc.make_error_with_kind(elvwasm::ErrorKinds::Invalid(format!("Part list not available err = {e}"))),
    };
    let pl:QPartList = serde_json::from_str(&s)?;

    let zip = GzEncoder::new(bw, flate2::Compression::default());
    let mut a = tar::Builder::new(zip);
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
        let mut header = tar::Header::new_gnu();
        header.set_size(usz as u64);
        header.set_cksum();
        BitcodeContext::log(&format!("zip starting {} part size = {usz}", part.hash.clone()));
        let b64_decoded = decode(&data.result)?;
        a.append_data(&mut header, part.hash.clone(), b64_decoded.as_slice())?;
    }
    a.finish()?;
    a.into_inner().unwrap().flush()?;
    BitcodeContext::log(&format!("Callback size = {total_size}"));
    bcc.callback(200, "application/zip", total_size)?;


    bcc.make_success_json(&json!(
        {
            "headers" : "application/zip",
            "body" : "SUCCESS",
            "result" : 0,
        }), id)
}


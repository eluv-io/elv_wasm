#![feature(try_trait_v2)]
#![feature(linked_list_cursors)]
extern crate base64;
extern crate elvwasm;
extern crate serde;
extern crate serde_json;
#[macro_use(defer)]
extern crate scopeguard;

use elvwasm::{
    implement_bitcode_module, jpc, register_handler, BitcodeContext, NewStreamResult, QPartList,
    SystemTimeResult,
};
use flate2::write::GzEncoder;
use serde_json::json;
use std::io::{BufWriter, ErrorKind, SeekFrom, Write};

implement_bitcode_module!("tar", do_tar_from_obj, "content", do_tar_from_obj);

#[derive(Debug)]
struct FabricWriter<'a> {
    bcc: &'a BitcodeContext,
    size: usize,
}

impl<'a> FabricWriter<'a> {
    fn new(bcc: &'a BitcodeContext, sz: usize) -> FabricWriter<'a> {
        FabricWriter { bcc, size: sz }
    }
}
impl<'a> std::io::Write for FabricWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        match self.bcc.write_stream("fos", buf) {
            Ok(s) => {
                self.bcc
                    .log_debug(&format!("Wrote {} bytes", buf.len()))
                    .unwrap_or_default(); // to gobble the log result
                let w: elvwasm::WritePartResult = serde_json::from_slice(&s)?;
                self.size += w.written;
                Ok(w.written)
            }
            Err(e) => Err(std::io::Error::new(ErrorKind::Other, e)),
        }
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        // Nothing to flush.  The BufWriter will handle its buffer independant using writes
        Ok(())
    }
}

impl<'a> std::io::Seek for FabricWriter<'a> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, std::io::Error> {
        match pos {
            SeekFrom::Start(s) => {
                self.bcc
                    .log_debug(&format!("SEEK from START {s}"))
                    .unwrap_or_default();
            }
            SeekFrom::Current(s) => {
                self.bcc
                    .log_debug(&format!("SEEK from CURRENT {s}"))
                    .unwrap_or_default();
            }
            SeekFrom::End(s) => {
                self.bcc
                    .log_debug(&format!("SEEK from END {s}"))
                    .unwrap_or_default();
            }
        }
        Ok(self.size as u64)
    }
}

#[no_mangle]
fn do_tar_from_obj(bcc: &mut elvwasm::BitcodeContext) -> CallResult {
    let http_p = &bcc.request.params.http;
    let qp = &http_p.query;
    let vqhot = &vec![bcc.request.q_info.qhot()];
    let obj_id = match qp.get("object_id_or_hash") {
        Some(x) => x,
        None => vqhot,
    };
    const DEF_CAP: usize = 50000000;
    let buf_cap = match qp.get("buffer_capacity") {
        Some(x) => {
            bcc.log_debug(&format!("new capacity of {x:?} set"))?;
            x[0].parse().unwrap_or(DEF_CAP)
        }
        None => DEF_CAP,
    };
    let total_size = 0;
    let mut fw = FabricWriter::new(bcc, total_size);
    {
        let bw = BufWriter::with_capacity(buf_cap, &mut fw);

        let pl: QPartList = bcc.q_part_list(obj_id[0].to_string()).try_into()?;

        let zip = GzEncoder::new(bw, flate2::Compression::default());
        let mut a = tar::Builder::new(zip);
        let time_cur: SystemTimeResult = bcc.q_system_time().try_into()?;
        for part in pl.part_list.parts {
            let stream_wm: NewStreamResult = bcc.new_stream().try_into()?;
            defer! {
                bcc.log_debug(&format!("Closing part stream {}", &stream_wm.stream_id)).unwrap_or_default();
                let _ = bcc.close_stream(stream_wm.stream_id.clone());
            }
            let _wprb = bcc.write_part_to_stream(
                stream_wm.stream_id.clone(),
                part.hash.clone(),
                bcc.request.q_info.hash.clone(),
                0,
                -1,
                false,
            )?;
            let usz = part.size.try_into()?;
            let data = bcc.read_stream(stream_wm.stream_id.clone(), usz)?;
            let mut header = tar::Header::new_gnu();
            header.set_size(usz as u64);
            header.set_cksum();
            header.set_mtime(time_cur.time);
            a.append_data(&mut header, part.hash.clone(), data.as_slice())?;
        }
        a.finish()?;
        let mut finished_writer = a.into_inner()?;
        finished_writer.flush()?;
    }
    bcc.log_debug(&format!("Callback size = {}", fw.size))?;
    bcc.callback(200, "application/zip", fw.size)?;

    bcc.make_success_json(&json!({}))
}

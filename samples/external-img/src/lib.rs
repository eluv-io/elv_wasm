#![feature(try_trait_v2)]
#![feature(linked_list_cursors)]
extern crate base64;
extern crate elvwasm;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use std::convert::TryInto;

use base64::{engine::general_purpose, Engine as _};
use elvwasm::{
    implement_bitcode_module, jpc, register_handler, BitcodeContext, ExternalCallResult,
    HttpParams, ReadStreamResult, SystemTimeResult,
};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::io::{BufWriter, ErrorKind, SeekFrom, Write};

use elvwasm::ErrorKinds;

implement_bitcode_module!("assets", do_assets);

const VERSION: &str = "1.0.7";
const MANIFEST: &str = ".download.info";

fn compute_image_url(operation: &str, meta: &serde_json::Value) -> CallResult {
    let v_represenations = meta.get("representations").ok_or(ErrorKinds::NotExist(
        "representations not found in meta".to_string(),
    ))?;
    let binding = meta
        .get("file")
        .ok_or(ErrorKinds::NotExist(
            "fabric_file not found in meta".to_string(),
        ))?
        .get("/")
        .ok_or(ErrorKinds::NotExist("failed to find /".to_string()))?
        .as_str()
        .ok_or(ErrorKinds::NotExist(
            "fabric_file not convertible to string".to_string(),
        ))?
        .to_string();
    let fabric_file: Vec<&str> = binding.split('/').collect();
    let offering = v_represenations
        .get(operation)
        .ok_or(ErrorKinds::NotExist(
            "operation not found in representations".to_string(),
        ))?
        .as_str()
        .ok_or(ErrorKinds::NotExist(
            "operation not convertible to string".to_string(),
        ))?;
    let file_path = &fabric_file[2..].join("/");
    let content_type = meta
        .get("attachment_content_type")
        .ok_or(ErrorKinds::NotExist(
            "attachment_content_type not found in meta".to_string(),
        ))?
        .as_str()
        .ok_or(ErrorKinds::NotExist(
            "attachment_content_type not convertible to string".to_string(),
        ))?;
    let url = format!("/image/{offering}/files/{file_path}");
    let jret = json!({
        "url": url,
        "content_type": content_type,
        "offering": offering,
    });
    Ok(jret.to_string().as_bytes().to_vec())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SummaryElement {
    asset: String,
    status: String,
}

fn process_multi_entry(
    bcc: &BitcodeContext,
    http_p: &HttpParams,
    qp: &HashMap<String, Vec<String>>,
    asset: &str,
    hash: &str,
    filename: &mut String,
) -> CallResult {
    bcc.log_debug(&format!("Bulk download param: {0}", hash))?;

    let meta: serde_json::Value = serde_json::from_slice(&bcc.sqmd_get_json_external(
        &bcc.request.q_info.qlib_id,
        hash,
        &format!("/assets/{asset}"),
    )?)?;
    *filename = meta
        .get("title")
        .ok_or(ErrorKinds::NotExist("title not found in meta".to_string()))?
        .as_str()
        .ok_or(ErrorKinds::Invalid(
            "title not convertible to string".to_string(),
        ))?
        .to_string();

    let result: ComputeCallResult = compute_image_url("download", &meta).try_into()?;
    bcc.log_debug(&format!("Compute call result = {0:?} ", result))?;
    get_single_offering_image(bcc, qp, &http_p.client_ip, &result.url, hash)
}

fn do_bulk_download(
    bcc: &BitcodeContext,
    http_p: &HttpParams,
    qp: &HashMap<String, Vec<String>>,
) -> CallResult {
    const HASH: usize = 2; // location in param vector
    bcc.log_debug("do_bulk_download")?;
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

        //let zip = GzEncoder::new(bw, flate2::Compression::default());
        let mut a = tar::Builder::new(bw);
        let time_cur: SystemTimeResult = bcc.q_system_time().try_into()?;
        let rsr: ReadStreamResult = bcc.read_stream("fis".to_string(), 0).try_into()?;

        let params: Vec<String> = if !rsr.result.is_empty() {
            let b64_decoded = general_purpose::STANDARD.decode(&rsr.result)?;

            let p: serde_json::Value = serde_json::from_slice(&b64_decoded)?;
            p.as_array()
                .ok_or(ErrorKinds::Invalid("params not an array".to_string()))?
                .iter()
                .map(|value| value.as_str().unwrap_or_default().to_string())
                .collect()
        } else {
            bcc.request
                .params
                .http
                .body
                .as_array()
                .map(|array| {
                    array
                        .iter()
                        .map(|value| value.as_str().unwrap_or_default().to_string())
                        .collect()
                })
                .unwrap_or_default()
        };
        bcc.log_debug(&format!("Bulk download params: {params:?}"))?;
        let mut v_file_status: Vec<SummaryElement> = vec![];

        for p in &params {
            let current_params = p.split('/').collect::<Vec<&str>>();
            let asset = current_params[current_params.len() - 1];
            let filename: &mut String = &mut "".to_string();
            let exr: ExternalCallResult =
                match process_multi_entry(bcc, http_p, qp, asset, current_params[HASH], filename) {
                    Ok(exr) => exr.try_into()?,
                    Err(e) => {
                        v_file_status.push(SummaryElement {
                            asset: format!("{0} Error={e}", p),
                            status: "failed".to_string(),
                        });
                        bcc.log_error(&format!("Error processing {p} : {e}"))?;
                        continue;
                    }
                };

            v_file_status.push(SummaryElement {
                asset: filename.to_string(),
                status: "success".to_string(),
            });
            let mut header = tar::Header::new_gnu();
            let b64_decoded = general_purpose::STANDARD.decode(&exr.fout)?;
            header.set_size(b64_decoded.len() as u64);
            header.set_cksum();
            header.set_mtime(time_cur.time);
            header.set_mode(0o644);
            a.append_data(&mut header, &filename, b64_decoded.as_slice())?;
        }
        let mut header = tar::Header::new_gnu();
        let contents = v_file_status
            .iter()
            .map(|x| format!("{0} {1}", x.asset, x.status))
            .collect::<Vec<String>>()
            .join("\n");
        header.set_size(contents.len() as u64);
        header.set_cksum();
        header.set_mtime(time_cur.time);
        header.set_mode(0o644);
        a.append_data(&mut header, MANIFEST, std::io::Cursor::new(contents))?;
        a.finish()?;
        let mut finished_writer = a.into_inner()?;
        finished_writer.flush()?;
    }
    bcc.log_debug(&format!("Callback size = {}", fw.size))?;
    bcc.callback(200, "application/tar", fw.size)?;
    bcc.make_success_json(&json!({}))
}

fn do_single_asset(
    bcc: &BitcodeContext,
    http_p: &HttpParams,
    qp: &HashMap<String, Vec<String>>,
    path_vec: Vec<&str>,
) -> CallResult {
    bcc.log_debug("do_single_asset")?;
    let asset = path_vec[path_vec.len() - 1];
    let operation = path_vec[2];
    let meta: serde_json::Value =
        serde_json::from_slice(&bcc.sqmd_get_json(&format!("/assets/{asset}"))?)?;
    let result: ComputeCallResult = compute_image_url(operation, &meta).try_into()?;

    let exr: ExternalCallResult = get_single_offering_image(
        bcc,
        qp,
        &http_p.client_ip,
        &result.url,
        &bcc.request.q_info.hash,
    )
    .try_into()?;
    let imgbits = &general_purpose::STANDARD.decode(&exr.fout)?;
    bcc.log_debug(&format!(
        "imgbits decoded size = {} fout size = {}",
        imgbits.len(),
        exr.fout.len()
    ))?;
    let mut filename = meta
        .get("title")
        .ok_or(ErrorKinds::NotExist("title not found in meta".to_string()))?
        .as_str()
        .ok_or(ErrorKinds::Invalid(
            "title not convertible to string".to_string(),
        ))?
        .to_string();
    let ct = meta
        .get("attachment_content_type")
        .ok_or(ErrorKinds::NotExist(
            "attachment_content_type not found in meta".to_string(),
        ))?
        .as_str()
        .ok_or(ErrorKinds::NotExist(
            "attachment_content_type not convertible to string".to_string(),
        ))?
        .to_string();
    let is_document = ct == "application/pdf";
    if ct != "image/jpeg" && !is_document && exr.format[0] == "image/jpeg" {
        filename += ".jpg"
    }
    bcc.log_debug(&format!(
        "RepAssets op={operation} asset={asset} isDoc={is_document} ct={ct} filename={filename}, rep image path={0} version={VERSION}, rep_image format={1}",result.url, &exr.format[0]
    ))?;
    if operation == "download" {
        let content_disp = format!("attachment; filename=\"{}\"", filename);
        bcc.callback_disposition(200, &exr.format[0], imgbits.len(), &content_disp, VERSION)?;
    }
    if operation == "preview" {
        if is_document {
            bcc.callback(200, &ct, imgbits.len())?;
        } else {
            bcc.callback(200, &exr.format[0], imgbits.len())?;
        }
    }
    bcc.write_stream("fos", imgbits)?;
    bcc.make_success_json(&json!({}))
}

fn get_single_offering_image(
    bcc: &BitcodeContext,
    qp: &HashMap<String, Vec<String>>,
    client_ip: &str,
    url: &str,
    hash: &str,
) -> CallResult {
    let params = json!({
        "http" : {
            "verb" : "GET",
            "headers": {
                "Content-type": [
                    "application/json"
                ]
            },
            "path" : url,
            "query" : qp,
            "client_ip" : client_ip,
        },
    });
    bcc.log_debug(&format!(
        "Calling external bitcode with params = {params:?}"
    ))?;
    bcc.call_external_bitcode("image", &params, hash, "builtin")
}

#[no_mangle]
fn do_assets(bcc: &mut BitcodeContext) -> CallResult {
    let http_p = &bcc.request.params.http;
    let qp = &http_p.query;
    let path_vec: Vec<&str> = bcc.request.params.http.path.split('/').collect();
    bcc.log_debug(&format!(
        "In Assets path_vec = {path_vec:?} http params = {http_p:?}"
    ))?;
    if path_vec[2] == "bulk_download" {
        do_bulk_download(bcc, http_p, qp)
    } else {
        do_single_asset(bcc, http_p, qp, path_vec)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ComputeCallResult {
    pub url: String,
    pub content_type: String,
    pub offering: String,
}

impl TryFrom<CallResult> for ComputeCallResult {
    type Error = Box<dyn std::error::Error + Sync + Send + 'static>;
    fn try_from(
        cr: CallResult,
    ) -> Result<ComputeCallResult, Box<dyn std::error::Error + Sync + Send + 'static>> {
        Ok(serde_json::from_slice(&cr?)?)
    }
}

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
            SeekFrom::Start(_s) => {}
            SeekFrom::Current(_s) => {}
            SeekFrom::End(_s) => {}
        }
        Ok(self.size as u64)
    }
}

#[test]
fn test_image_url_generation() {
    let meta = json!({
      "asset_type": "image",
      "attachment_content_type": "image/jpeg",
      "attachment_file_name": "AquarionEVOL_ShowHeroTablet.jpg",
      "attachment_file_size": 1285603,
      "attachment_updated_at": "2024-02-29T02:54:03.510Z",
      "file": {
        ".": {
          "auto_update": {
            "tag": "latest"
          },
          "container": "hq__J6aApMvBZPCkikMMMh9yZ69ymkr6c2DcmFj39Wk5GcQXJTd5z4VG7RgbiKHehPvHc3Q92naTQg"
        },
        "/": "./files/assets/11e1e-45d4a-06e3-6efc76.jpg"
      },
      "image": {
        "height": 2394,
        "orientation": "portrait",
        "width": 1728
      },
      "original_access": 400,
      "representations": {
        "download": "tier1",
        "preview": "none",
        "thumbnail": "tier1"

      },
      "title": "AquarionEVOL_ShowHeroTablet.jpg",
      "uuid": "11e1e-45d4a-06e3-6efc76",
      "version": "2",
    });
    let operation = "download";
    let result: ComputeCallResult = compute_image_url(operation, &meta).try_into().unwrap();
    assert_eq!(
        result.url,
        "/image/tier1/files/assets/11e1e-45d4a-06e3-6efc76.jpg"
    );
    let operation = "preview";
    let result: ComputeCallResult = compute_image_url(operation, &meta).try_into().unwrap();
    assert_eq!(
        result.url,
        "/image/none/files/assets/11e1e-45d4a-06e3-6efc76.jpg"
    );
    let operation = "thumbnail";
    let result: ComputeCallResult = compute_image_url(operation, &meta).try_into().unwrap();
    assert_eq!(
        result.url,
        "/image/tier1/files/assets/11e1e-45d4a-06e3-6efc76.jpg"
    );

    assert_eq!(result.content_type, "image/jpeg")
}

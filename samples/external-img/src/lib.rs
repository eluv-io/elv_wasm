extern crate base64;
extern crate elvwasm;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use base64::{engine::general_purpose, Engine as _};
use elvwasm::BitcodeContext;
use elvwasm::ErrorKinds;
use elvwasm::{implement_bitcode_module, jpc, register_handler, ExternalCallResult};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;

implement_bitcode_module!("assets", do_assets);

const VERSION: &str = "1.0.7";

fn compute_image_url(operation: &str, meta: &serde_json::Value) -> CallResult {
    let v_represenations = meta.get("representations").ok_or(ErrorKinds::NotExist(
        "representations not found in meta".to_string(),
    ))?;
    let binding = meta
        .get("file")
        .clone()
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
    let fabric_file: Vec<&str> = binding.split("/").collect();
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

#[no_mangle]
fn do_assets(bcc: &mut BitcodeContext) -> CallResult {
    let http_p = &bcc.request.params.http;
    let qp = &http_p.query;
    let path_vec: Vec<&str> = bcc.request.params.http.path.split('/').collect();
    let asset = path_vec[path_vec.len() - 1];
    let operation = path_vec[2];
    let meta: serde_json::Value =
        serde_json::from_slice(&bcc.sqmd_get_json(&format!("/assets/{asset}"))?)?;
    let result: ComputeCallResult = compute_image_url(operation, &meta).try_into()?;
    let params = json!({
        "http" : {
            "verb" : "GET",
            "headers": {
                "Content-type": [
                    "application/json"
                ]
            },
            "path" : result.url,
            "query" : qp,
            "client_ip" : http_p.client_ip,
        },
    });
    let exr: ExternalCallResult = bcc
        .call_external_bitcode("image", &params, &bcc.request.q_info.hash, "builtin")
        .try_into()?;
    bcc.log_info("here")?;
    let imgbits = &general_purpose::STANDARD.decode(&exr.fout)?;
    console_log(&format!(
        "imgbits decoded size = {} fout size = {}",
        imgbits.len(),
        exr.fout.len()
    ));
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
    if ct != "image/jpeg" && !is_document && exr.format == "image/jpeg" {
        filename += ".jpg"
    }
    bcc.log_debug(&format!(
        "RepAssets op={operation} asset={asset} isDoc={is_document} ct={ct} filename={filename}, rep image path={0} version={VERSION}",result.url
    ))?;
    if operation == "download" {
        let content_disp = format!("attachment; filename=\"{}\"", filename);
        bcc.callback_disposition(200, &ct, imgbits.len(), &content_disp, VERSION)?;
    }
    if operation == "preview" {
        if is_document {
            bcc.callback(200, &ct, imgbits.len())?;
        } else {
            bcc.callback(200, "image/jpeg", imgbits.len())?;
        }
    }
    bcc.write_stream("fos", &imgbits)?;
    bcc.make_success_json(&json!({}))
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

extern crate elvwasm;
extern crate serde_json;
#[macro_use(defer)] extern crate scopeguard;
use serde_json::json;
use serde::{Deserialize, Serialize};

extern crate image;
use image::GenericImageView;
use image::{jpeg::JpegEncoder};

use elvwasm::{implement_bitcode_module, jpc, register_handler, BitcodeContext, NewStreamResult, ReadStreamResult};

implement_bitcode_module!("image", do_img);

#[derive(Serialize, Deserialize,  Clone, Debug, Default)]
pub struct WatermarkJson {
  #[serde(default)]
  pub x: String,
  #[serde(default)]
  pub y: String,
  #[serde(default)]
  pub image: String,
}

#[derive(Serialize, Deserialize,  Clone, Debug)]
pub struct ImageWatermark{
  #[serde(default)]
  pub image_watermark : WatermarkJson
}

#[derive(Serialize, Deserialize,  Clone, Debug)]
pub struct WriteResult{
  #[serde(default)]
  pub written : usize
}

fn parse_asset(path:&str)-> String{
    let mut pos:Vec<&str> = path.split('/').collect();
    if pos.len() > 2{
      pos = pos[3..].to_owned();
      return pos.join("/");
    }
    "".to_owned()
}

fn get_offering(bcc :&BitcodeContext, input_path:&str) -> CallResult {
    let v:Vec<&str> = input_path.split('/').collect();
    let mut s = "";
    if v.len() > 1 {
      s = v[2];
    }
    let json_path = format!("/public/image/offerings/{}",s);
    // input_path should just be offering
    bcc.sqmd_get_json(&json_path)
}

fn do_img<>(bcc: &mut elvwasm::BitcodeContext<>) -> CallResult {
    let http_p = &bcc.request.params.http;
    let offering = get_offering(bcc, &http_p.path)?;
    let offering_json:ImageWatermark = serde_json::from_slice(&offering)?;
    let asset_path = parse_asset(&http_p.path);
    BitcodeContext::log(&format!("offering = {} asset_path = {}", &offering_json.image_watermark.image, &asset_path));
    let res = bcc.new_stream()?;
    let stream_main: NewStreamResult = serde_json::from_slice(&res)?;
    defer!{
        let _ = bcc.close_stream(stream_main.stream_id.clone());
    }
    let f2s = bcc.q_file_to_stream(&stream_main.stream_id, &asset_path, &bcc.request.q_info.hash)?;
    let written: WriteResult = serde_json::from_slice(&f2s)?;
    BitcodeContext::log(&format!("written = {}", &written.written));
    let read_res = bcc.read_stream(stream_main.stream_id.clone(), written.written)?;
    let read_data: ReadStreamResult = serde_json::from_slice(&read_res)?;
    let base = read_data.result;
    let buffer = base64::decode(base)?;
    BitcodeContext::log(&format!("bytes read = {}", read_data.retval));
    let img = image::load_from_memory_with_format(&buffer, image::ImageFormat::Jpeg)?;
    let (h,w) = img.dimensions();
    let height_str = &http_p.query["height"];
    let height: usize = height_str[0].parse().unwrap_or(0);
    let width_factor: f32 = h as f32/w as f32;
    let new_width:usize = (width_factor*height as f32) as usize;
    let br = img.resize( new_width as u32, height as u32, image::imageops::FilterType::Lanczos3);
    let mut bytes: Vec<u8> = Vec::new();
    let mut encoder = JpegEncoder::new(&mut bytes);
    encoder.encode(&br.to_bytes(), new_width as u32, height as u32, br.color())?;
    bcc.callback(200, "image/jpeg", bytes.len())?;
    BitcodeContext::write_stream_auto(bcc.request.id.clone(), "fos", &bytes)?;

    bcc.make_success_json(&json!(
      {
          "headers" : "application/json",
          "body" : "SUCCESS",
          "result" : 0,
      }), &bcc.request.id)
}
extern crate elvwasm;
extern crate serde_json;
#[macro_use(defer)] extern crate scopeguard;
use serde_json::json;
use serde_derive::{Deserialize, Serialize};




use elvwasm::*;

implement_bitcode_module!("image", do_image);

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
  let json_path = format!("/public/image/offerings/{s}");
  // input_path should just be offering
  bcc.sqmd_get_json_resolve(&json_path)
}

#[derive(Serialize, Deserialize,  Clone, Debug, Default)]
pub struct WatermarkJson {
  #[serde(default)]
  pub x: String,
  #[serde(default)]
  pub y: String,
  #[serde(default)]
  pub image: String,
  #[serde(default)]
	pub height:String,
  #[serde(default)]
	pub opacity:String,
}

#[derive(Serialize, Deserialize,  Clone, Debug)]
pub struct ImageWatermark{
  #[serde(default)]
  pub image_watermark : WatermarkJson
}

fn fabric_file_to_tmp_file(bcc :&BitcodeContext,fabric_file:&str,temp_file:&str) -> CallResult {
  if fabric_file.is_empty() || temp_file.is_empty(){
    return bcc.make_error_with_kind(ErrorKinds::Invalid("parameters must not be empty strings".to_string()));
  }
  let input = fabric_file.to_string();
  let output = temp_file;
  BitcodeContext::log(&format!("input={input}"));
  let j = json!({
    "stream_id":output,
    "path":input,
    "hash_or_token": bcc.request.q_info.hash,
  });
  bcc.call_function("QFileToStream", j, "core")?;
  bcc.close_stream(output.to_string())?;
  bcc.make_success("DONE")
}

fn ffmpeg_run_no_watermark(bcc:&BitcodeContext, height:&str,input_file:&str, new_file:&str) -> CallResult {
  BitcodeContext::log("ffmpeg_run_no_watermark");
  let scale_factor = &format!("scale={height}:-1");
  // need to run ffmpeg here input file is in input_file
  let mut ffmpeg_args_no_watermark = [
      "-hide_banner",
      "-nostats",
      "-y",
      "-i", "REPLACEME",
      "-vf","REPLACEME",
      "-f", "singlejpeg",
      "REPLACEME",
  ];
  ffmpeg_args_no_watermark[4] = input_file;
  ffmpeg_args_no_watermark[6] = scale_factor;
  ffmpeg_args_no_watermark[9] = new_file;
  bcc.ffmpeg_run(ffmpeg_args_no_watermark.to_vec())
}

fn ffmpeg_run_watermark(bcc:&BitcodeContext, height:&str, input_file:&str, new_file:&str, watermark_file:&str, overlay_x:&str, overlay_y:&str) -> CallResult{
  let base_placement = format!("{overlay_x}:{overlay_y}");
  let scale_factor = "[0:v]scale=%SCALE%:-1[bg];[bg][1:v]overlay=%OVERLAY%";
  let scale_factor = &scale_factor.replace("%SCALE%", height).replace("%OVERLAY%", &base_placement);
  if input_file.is_empty() || watermark_file.is_empty() || new_file.is_empty(){
    return bcc.make_error_with_kind(ErrorKinds::Invalid("parameter validation failed, one file is empty or null".to_string()));
  }
  // need to run ffmpeg here input file is in input_file
  let ffmpeg_args = ["-hide_banner","-nostats","-y","-i", input_file,"-i", watermark_file,"-filter_complex", scale_factor,"-f", "singlejpeg", new_file].to_vec();

  bcc.ffmpeg_run(ffmpeg_args)
}

fn do_image<>(bcc: &mut elvwasm::BitcodeContext) -> CallResult {
  BitcodeContext::log("HELLO FROM do image");
  let http_p = &bcc.request.params.http;
  let qp = http_p.query.clone();
  BitcodeContext::log(&format!("In do_image hash={} headers={:#?} query params={qp:#?}",&bcc.request.q_info.hash, &http_p.headers));
  let offering = get_offering(bcc, &http_p.path)?;
  let offering_json:WatermarkJson = serde_json::from_slice(&offering)?;
  let id = bcc.request.id.clone();
  let ifs = bcc.new_file_stream()?;
  let wfs = bcc.new_file_stream()?;
  let ofs = bcc.new_file_stream()?;
  let input_file_stream:FileStream = serde_json::from_slice(&ifs)?;
  let watermark_file_stream:FileStream = serde_json::from_slice(&wfs)?;
  let output_file_stream:FileStream = serde_json::from_slice(&ofs)?;
  defer!{
    let _ = bcc.close_stream(input_file_stream.stream_id.clone());
    let _ = bcc.close_stream(watermark_file_stream.stream_id.clone());
    let _ = bcc.close_stream(output_file_stream.stream_id.clone());
  }
  let asset_path = parse_asset(&http_p.path);
  fabric_file_to_tmp_file(bcc, &asset_path, &input_file_stream.stream_id)?;
  if !offering_json.image.is_empty() {
    if watermark_file_stream.stream_id.is_empty() || watermark_file_stream.file_name.is_empty(){
      return bcc.make_error_with_kind(ErrorKinds::NotExist(format!("failed to acquire watermark stream path = {asset_path}")));
    }
    fabric_file_to_tmp_file(bcc, &offering_json.image, &watermark_file_stream.stream_id)?;
    ffmpeg_run_watermark(bcc, &offering_json.height,
                                       &input_file_stream.file_name.clone(), &output_file_stream.file_name.clone(),
                                       &watermark_file_stream.file_name.clone(), &offering_json.x, &offering_json.y)?;
  }else{
    ffmpeg_run_no_watermark(bcc, &qp["height"][0], &input_file_stream.file_name,&output_file_stream.file_name)?;
  }
  let sz_string = bcc.file_stream_size(&output_file_stream.file_name);
  bcc.callback(200, "image/jpeg", sz_string)?;
  bcc.file_to_stream(&output_file_stream.file_name, "fos")?;
  bcc.make_success_json(&json!({"body" : "SUCCESS"}), &id)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

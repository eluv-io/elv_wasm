import {
    register,
    handleCall,
    hostCall,
    handleAbort,
    consoleLog,
  } from "../assembly";

  import { JSON } from "assemblyscript-json";
  import {
    BitcodeContext,
    QuoteString,
    registerHandler,
    elv_return_type,
    Error,
    _jpc
  } from "../include/bitcode-context";




  // This must be present in the entry file.
export function __guest_call(operation_size: usize, payload_size: usize): bool {
  registerHandler("image", doImage)
  register("_jpc", _jpc)
  return handleCall(operation_size, payload_size);
}

function parse_asset(path:string):string{
  let pos = path.split('/');
  if (pos.length > 2){
    pos = pos.slice(3, pos.length);
    return pos.join("/");
  }
  return "";
}
class Offering{
  x:JSON.Str;
  y:JSON.Str;
  image: JSON.Str;
  err:Error;
  constructor(){
    this.x = <JSON.Str>JSON.from("");
    this.y = <JSON.Str>JSON.from("");
    this.image = <JSON.Str>JSON.from("");
    this.err = new Error("");
  }
}

function GetWatermarkOffering(offering:ArrayBuffer) : Offering | null {
  consoleLog("GetWatermarkOffering =");
  let j = <JSON.Obj>JSON.parse(String.UTF8.decode(offering));
  let ret = new Offering();
  let jwm = j.getObj("image_watermark");
  if (jwm == null){
    return null;
  }
  if (jwm != null){
    consoleLog("Found image watermark");
    let imageStr: JSON.Str | null = jwm.getString("image");
    if (imageStr == null){
      consoleLog("image not found");
      return null;
    }
    ret.image = imageStr;
    let x : JSON.Str | null = jwm.getString("x");
    if (x == null){
      consoleLog("x not found");
      return null;
    }
    ret.x = x;
    let y : JSON.Str | null = jwm.getString("y");
    if (y == null){
      consoleLog("y not found");
      return null;
    }
    ret.y = y;
  }
  consoleLog("returning offering image="+ret.image.toString());
  return ret;
}

function GetOffering(bcc :BitcodeContext, input_path:string) : elv_return_type{
  let v = input_path.split("/");
  let s = "";
  if (v.length > 1)
      s = v[2];
  let json_path = "/public/image/offerings/" + s;
  // input_path should just be offering
  let ret = bcc.SQMDGetJSON(json_path);
  if (ret.isError()){
    return ret;
  }
  return bcc.SQMDGetJSON(json_path);
}

function FabricFileToTempFile(bcc :BitcodeContext,fabric_file:JSON.Str|null,temp_file:string):elv_return_type {
  if (fabric_file == null || temp_file == null){
    let ert = new elv_return_type();
    ert._1 = new Error("input files may not be null");
    return ert;
  }
  let input = fabric_file.toString();
  let output = temp_file;
  consoleLog("input="+input);
  let j = new JSON.Obj();
  j.set<string>("stream_id",output);
  j.set<string>("path",input);
  j.set<string>("hash_or_token", bcc.hash);
  let ret = bcc.Call("QFileToStream", j, "core");
  if (ret.isError()){
    return bcc.make_error_closer("QFileToStream failed", ret._1,input);
  }
  bcc.CloseStream(output);
  return bcc.make_success(String.UTF8.encode("DONE"));
}

function FFmpegRunNoWatermark(bcc:BitcodeContext, height:string,input_file:string, new_file:string):elv_return_type {
  consoleLog("FFmpegRunNoWatermark");
  let scale_factor = "scale=%d:-1".replace("%d", height);
  // need to run ffmpeg here input file is in input_file
  let ffmpeg_args_no_watermark = [
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
  return bcc.FFMPEGRun(ffmpeg_args_no_watermark);
}

function FFmpegRunWatermark(bcc:BitcodeContext, height:string, input_file:string, new_file:string, watermark_file:string, overlay_x:string, overlay_y:string):elv_return_type{
  //auto base_placement = "(main_w-overlay_w)/2:(main_h-overlay_h)/2";
  let base_placement = overlay_x+":"+overlay_y;
  let scale_factor = "[0:v]scale=%SCALE%:-1[bg];[bg][1:v]overlay=%OVERLAY%";
  scale_factor = scale_factor.replace("%SCALE%", height).replace("%OVERLAY%", base_placement);
  if (input_file == "" || watermark_file == "" || new_file == ""){
    let msg = "parameter validation failed, one file is empty or null";
    return bcc.make_error(msg, new Error(msg));
  }
  // need to run ffmpeg here input file is in input_file
  let ffmpeg_args = ["-hide_banner","-nostats","-y","-i", input_file,"-i", watermark_file,"-filter_complex", scale_factor,"-f", "singlejpeg", new_file];

  return bcc.FFMPEGRun(ffmpeg_args);
}

var qpret = new Map<string,string>();
function doImage(payload : BitcodeContext) : ArrayBuffer {
  consoleLog("doImage");

  let jpcParams: JSON.Obj = <JSON.Obj>(payload.jpcParams.getObj("params"));
  consoleLog("params="+jpcParams.toString());
  if (jpcParams == null){
    return payload.ReturnErrorBuffer("could not find params");
  }

  let httpParams = <JSON.Obj>jpcParams.getObj("http");
  let qpret = payload.QueryParams(jpcParams);
  if (qpret.size == 0){
    consoleLog("no query parameters found");
  }

  let heightString = qpret.get("height");
  consoleLog("heightString ="+heightString);
  let path =  (<JSON.Str>httpParams.getString("path")).toString();
  let offering = GetOffering(payload,path);
  if (offering.isError()){
    return payload.ReturnErrorBuffer("failed to get offering");
  }
  let assetPath = parse_asset(path);
  consoleLog("assetPath="+assetPath);
  let watermark = GetWatermarkOffering(offering.getBuffer());
  let inputFileStream = payload. NewFileStream();
  let input = inputFileStream.get("stream_id");
  let inputFile = inputFileStream.get("file_name");

  if (input == "" || inputFile == ""){
    return payload.ReturnErrorBuffer("failed to stream_id");
  }
  let watermarkJsonStream = payload.NewFileStream();
  let watermarkStream = watermarkJsonStream.get("stream_id");
  let watermarkFile = watermarkJsonStream.get("file_name");
  let outputJsonStream = payload.NewFileStream();
  let outputStream = outputJsonStream.get("stream_id");
  let outputFilename = outputJsonStream.get("file_name");
  if (outputStream == "" || outputFilename == ""){
    return payload.ReturnErrorBuffer("failed to acquire output stream");
  }

  let image_ret = FabricFileToTempFile(payload, new JSON.Str(assetPath), input);
  if (image_ret.isError()){
      return payload.ReturnErrorBuffer("failed to get fabric image:"+ assetPath)
  }
  let outFile = outputFilename.toString();
  if (watermark != null){
    if (watermarkStream == "" || watermarkFile == ""){
      return payload.ReturnErrorBuffer("failed to acquire watermark stream");
    }
    let image_ret = FabricFileToTempFile(payload,  watermark.image, watermarkStream);
    if (image_ret.isError()){
        return payload.ReturnErrorBuffer("failed to get watermark image");
    }
    let run_ret = FFmpegRunWatermark(payload, heightString,inputFile.toString(), outFile, watermarkFile.toString(), watermark.x.toString(), watermark.y.toString());
    if (run_ret.isError()){
      return payload.ReturnErrorBuffer("ffmpegRun failed");
    }

  }else{
    let run_ret = FFmpegRunNoWatermark(payload, heightString, inputFile.toString(),outFile);
    if (run_ret.isError()){
      return payload.ReturnErrorBuffer("ffmpegRun failed");
    }
  }
  let sz = payload.FileStreamSize(outFile);
  let p = payload.Callback(200, "image/jpeg", <i32>sz);
  if (p.isError()){
    return payload.ReturnErrorBuffer("Callback failed");
  }
  let fswRet = payload.FileToStream(outFile, "fos");

  consoleLog("done-ish sz="+ sz.toString());
  return payload.ReturnSuccessBuffer("{\"body\" : \"SUCCESS\"}");
}

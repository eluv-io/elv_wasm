import {
    register,
    handleCall,
    hostCall,
    handleAbort,
    consoleLog,
  } from "../assembly";

  import { JSON, JSONDecoder, JSONEncoder, JSONHandler } from "assemblyscript-json";
  import {
    BitcodeContext,
    registerHandler,
    _jpc,
  } from "../include/bitcode-context";


  register("_JPC", _jpc)
  registerHandler("content", doProxy)


  // This must be present in the entry file.
export function __guest_call(operation_size: usize, payload_size: usize): bool {
  return handleCall(operation_size, payload_size);
}

function replaceMapInString(m:Map<string,string>, s:string):string{
  let retval:string = s;
  for (let i =0; i < m.keys().length; i++){
    let key = m.keys()[i];
    retval = retval.replaceAll("${"+key+"}", m.get(key));
  }
  return retval;
}

function doProxy(bcc : BitcodeContext) : ArrayBuffer {
  //consoleLog("params="+bcc.jpcParams.toString());
  let httpParams = bcc.jpcParams.getObj("params");
  if (httpParams == null){
    return bcc.ReturnErrorBuffer("No http params found");
  }
  let qpret = bcc.QueryParams(httpParams);
  let ret = bcc.SQMDGetJSON("/request_parameters");
  if (ret.isError()){
    return bcc.ReturnErrorBuffer("failed to get request_parameters");
  }
  let jsonToExpand = String.UTF8.decode(ret.getBuffer());
  let expandedJson = replaceMapInString(qpret, jsonToExpand);
  let requestString = `{"request":`.concat(expandedJson).concat("}");

  let callParams = JSON.parse(requestString);
  ret =  bcc.ProxyHttp(callParams);
  if (ret.isError()){
    return bcc.ReturnErrorBuffer("Proxy failure");
  }
  let retBuf = ret.getBuffer();
  let tempRet = bcc.Callback(200, "application/json", retBuf.byteLength);
  if (tempRet.isError()){
    return bcc.ReturnErrorBuffer("Callback failure");
  }
  let dec = String.UTF8.decode(retBuf);
  let j = <JSON.Obj>JSON.parse(dec);
  let vRes : JSON.Value | null = j.get("result");
  if (vRes != null){
      tempRet = bcc.WriteStream("fos", String.UTF8.encode(vRes.toString()), -1);
      if (tempRet.isError()){
        return bcc.ReturnErrorBuffer("WriteStream failure");
      }
  }else{
    tempRet = bcc.WriteStream("fos", retBuf, -1);
    if (tempRet.isError()){
      return bcc.ReturnErrorBuffer("WriteStream failure");
    }
  }
  return bcc.ReturnSuccessBuffer(`{"body" : "SUCCESS"}`);

}

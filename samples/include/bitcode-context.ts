import {
  register,
  handleCall,
  hostCall,
  handleAbort,
  consoleLog,
} from "../assembly";

import { JSON, JSONDecoder, JSONEncoder, JSONHandler } from "assemblyscript-json";

export type CallFunction = (payload: BitcodeContext) => ArrayBuffer

var handlerFunctions = new Map<string, CallFunction>()

export function registerHandler(operation: string, fn: CallFunction): void {
  handlerFunctions.set(operation, fn)
}

function eachEl(val:string, i:i32, ar:Array<string>): void{
  consoleLog("ELEMENT="+val);
}

function getFunction(name: string): CallFunction | null {
  consoleLog("SIZE="+handlerFunctions.size.toString())
  let keys = handlerFunctions.keys()
  keys.forEach(eachEl);
  // for (let key of Array.from(handlerFunctions.keys())) {
  //   consoleLog("KEY="+key);
  // }
  if (!handlerFunctions.has(name)) {
    return null
  }
  return handlerFunctions.get(name)
}

  // Abort function
  function abort(message: string | null, fileName: string | null, lineNumber: u32, columnNumber: u32): void {
    handleAbort(message, fileName, lineNumber, columnNumber)
  }


  /**
   * Converts an ArrayBuffer to a String.
   *
   * @param buffer - Buffer to convert.
   * @returns String.
   */
  function arrayBufferToString(buffer: ArrayBuffer): string {
    return String.fromCharCode.apply(null, Array.from(new Uint16Array(buffer)));
  }


  /**
   * Converts a String to an ArrayBuffer.
   *
   * @param str - String to convert.
   * @returns ArrayBuffer.
   */
  function stringToArrayBuffer(str: string): ArrayBuffer {
    const stringLength = str.length;
    const buffer = new ArrayBuffer(stringLength * 2);
    const bufferView = new Uint16Array(buffer);
    for (let i = 0; i < stringLength; i++) {
      bufferView[i] = str.charCodeAt(i);
    }
    return buffer;
  }

type kind = string;
type error = string;

export function _jpc(payload: ArrayBuffer): ArrayBuffer {
  consoleLog("IN JPC handler");
  let payloadString = String.UTF8.decode(payload);
  consoleLog("payload="+payloadString);
  let payVal = JSON.parse(payloadString);
  consoleLog("payVal="+payVal.toString());
  let jsonObj: JSON.Obj | null = <JSON.Obj>(payVal);
  if (jsonObj == null){
    return BitcodeContext.ReturnErrorBuffer("Unable to parse payload", "0");
  }
  consoleLog("After parse");
  consoleLog("json parse="+jsonObj.toString());
  let idStr: JSON.Str | null = jsonObj.getString("id");
  if (idStr == null){
    return BitcodeContext.ReturnErrorBuffer("ID not found", "0");
  }
  let jpcStr: JSON.Str | null = jsonObj.getString("jpc");
  if (jpcStr == null){
    return BitcodeContext.ReturnErrorBuffer("jpc not found", idStr.toString());
  }

  let jpcParams: JSON.Obj = <JSON.Obj>(jsonObj.getObj("params"));
  consoleLog("params="+jpcParams.toString());
  if (jpcParams == null){
    return BitcodeContext.ReturnErrorBuffer("params not found", idStr.toString());
  }

  let httpParams: JSON.Obj = <JSON.Obj>(jpcParams.getObj("http"));
  consoleLog("http="+httpParams.toString());
  if (httpParams == null){
    return BitcodeContext.ReturnErrorBuffer("http params not found", idStr.toString());
  }

  let methodOrNull: JSON.Str | null = httpParams.getString("path");
  let method = "";
  if (methodOrNull == null){
    return BitcodeContext.ReturnErrorBuffer("no method provided", idStr.toString());
  }
  method = methodOrNull.valueOf();
  let sliceMethod = method.split("/");
  let pathLoc = 0;
  if (sliceMethod.length > 1)
    pathLoc = 1;
  let qinfoParams: JSON.Obj | null = jsonObj.getObj("qinfo");
  if (qinfoParams == null){
    return BitcodeContext.ReturnErrorBuffer("qinfo not provided", idStr.toString());
  }
  let methodName = sliceMethod[pathLoc];
  consoleLog("method="+methodName+" count="+methodName.length.toString());
  let bcc = new BitcodeContext(jsonObj, hostCall, consoleLog, payload, idStr.toString());
  let fn = getFunction(methodName);
  if (fn != null){
     let ret = fn(bcc);
     bcc.cleanup();
     return ret;
  }
  else{
    let msg = "FAILURE method handler not found for: " + methodName;
    return BitcodeContext.ReturnErrorBuffer(msg, idStr.toString())
  }
}

class ErrorKinds{
    Other:kind = "unclassified error";
    NotImplemented:kind = "not implemented";
    Invalid:kind = "invalid";
    Permission:kind = "permission denied";
    IO:kind = "I/O error";
    Exist:kind = "item already exists";
    NotExist:kind = "item does not exist";
    IsDir:kind = "item is a directory";
    NotDir:kind = "item is not a directory";
    Finalized:kind = "item is already finalized";
    NotFinalized:kind = "item is not finalized";
    BadHttpParams:kind = "Invalid Http params specified";
};

export class Error extends ErrorKinds{
  constructor(p1:string, holder1: string = "", holder2: string = "",holder3: string = "", holder4: string = "", holder5: string = ""){
      super();
      this.Fields = new Map<string,string>();
      if (p1 == ""){
        consoleLog("in Error ctor");
        this._is_error = false;
      }else{
        this._is_error = true;
        this.Fields.set("op", p1);
        let placeholders = new Array<string>();
        placeholders.push(holder1);
        placeholders.push(holder2);
        placeholders.push(holder3);
        placeholders.push(holder4);
        placeholders.push(holder5);

        if (placeholders != null){
          this.init(placeholders);
        }
      }
  }
  init(rem:string[]) : void {
    this.Fields.set(rem[0],rem[1]);
    this.init(rem.slice(2));
  }
  toJSON() : JSONEncoder {
    let encoder = new JSONEncoder();
    encoder.pushObject("error");
    let k = this.Fields.keys();
    for(let i=0; i < k.length;++i) {
      //console.log(k[i]);
      let val = this.Fields.values();
      //console.log(val[i]);
      encoder.setString(k[i], val[i]);
    }
    encoder.popObject();
    return encoder;
  }
  unmarshalled:boolean;
  _is_error:boolean;
  Fields:Map<string,string> = new Map<string,string>();
}

export class elv_return_type {
  constructor(){
    this._0 = new ArrayBuffer(1);
    this._1 = new Error("");
    this._2 = null;
  }
  _0: ArrayBuffer; //result
  _1: Error; //error
  _2: JSON.Obj | null;

  isError() : boolean{
    return this._1._is_error
  }
  getBuffer() : ArrayBuffer{
    return this._0;
  }
  getError() : Error{
    return this._1;
  }
  getErrorString() : string{
    return this._1.toJSON().toString();
  }
  getJSON() : JSON.Obj {
    return <JSON.Obj>JSON.parse(String.UTF8.decode(this._0));
  }

};



export function QuoteString(s:string) : string{
  return "\""+s+"\"";
}
export class BitcodeContext{
    hash: string;
    id : string;
    write_token : string;
    qlib_id :string;
    qtype :string;
    jpcParams :JSON.Obj;
    payload : ArrayBuffer;
    openStreams : Array<string>;

    log : (message: string) => void ;
    call : (binding: string, namespace: string, operation: string, payload: ArrayBuffer) => ArrayBuffer;


    static ReturnErrorBuffer(msg:string, idStr:string) : ArrayBuffer {
      consoleLog("Return Error Buf="+msg);
      let err = new Error(msg);
      let strInnerRet = `{ "headers" : "application/json", "body" : __ERR__, "result":0}`.replace("__ERR__", err.toJSON().toString());
      let strRet = "{\"jpc\":\"1.0\", \"id\":\"$id\", \"error\" : $result}".replace("$id", idStr).replace("$result", strInnerRet);
      return String.UTF8.encode(strRet);
    }

    static ReturnSuccessBuffer(msg:string, idStr:string) : ArrayBuffer {
      let strRet = "{\"jpc\":\"1.0\", \"id\":\"$id\", \"result\" : $result}".replace("$id", idStr.toString()).replace("$result", msg);
      return String.UTF8.encode(strRet);
    }

    ReturnErrorBuffer(msg:string) : ArrayBuffer {
      return BitcodeContext.ReturnErrorBuffer(msg, this.id)
    }

    ReturnSuccessBuffer(msg:string) : ArrayBuffer {
      return BitcodeContext.ReturnSuccessBuffer(msg, this.id)
    }

    constructor(jpcParams : JSON.Obj, hostCall: (binding: string, namespace: string, operation: string, payload: ArrayBuffer) => ArrayBuffer, consoleLog : (message: string) => void, payload:ArrayBuffer, id:string ) {
          let qinfoParams: JSON.Obj | null = jpcParams.getObj("qinfo");
          this.log = consoleLog;
          this.call = hostCall;
          this.jpcParams = jpcParams;
          this.openStreams = new Array<string>();
          this.payload = payload;
          this.id = id;
          if (qinfoParams != null){
            let hashOrNull : JSON.Str | null = qinfoParams.getString("hash");
            this.hash = (hashOrNull != null) ? hashOrNull.toString() : "";
            let writeTokenOrNull : JSON.Str | null = qinfoParams.getString("write_token");
            this.write_token = (writeTokenOrNull != null) ? writeTokenOrNull.toString() : "";
            let libidOrNull : JSON.Str | null = qinfoParams.getString("qlib_id");
            this.qlib_id  = (libidOrNull != null) ? libidOrNull.toString() : "";
            let qtypeOrNull : JSON.Str | null = qinfoParams.getString("qtype");
            this.qtype = (qtypeOrNull != null) ? qtypeOrNull.toString() : "";
          }else{
            this.hash = "";
            this.write_token = "";
            this.qlib_id  = ""
            this.qtype = "";
          }
      }

      cleanup() : void {
        for (let i = 0; i < this.openStreams.length; i++){
          this.CloseStream(this.openStreams[i]);
        }
      }

      make_error(s:string, err:Error) : elv_return_type {
        let ret = new elv_return_type();
        ret._0 = String.UTF8.encode("{ \"error\" :"+s+"}");
        ret._1 = err;
        return ret;
      }
      make_error_closer(s:string, err:Error, streamName:string) : elv_return_type {
        let retClose = this.CloseStream(s);
        if (retClose.isError){
          return retClose;
        }
        return this.make_error(s,err);
      }
      make_success(response : ArrayBuffer) : elv_return_type {
        let ret = new elv_return_type();
        ret._0 = response;
        ret._1._is_error = false;
        consoleLog("done success");
        return ret;
      }

      WriteStream(streamToWrite:string,  src:ArrayBuffer, len:i32) : elv_return_type {
        if (len == -1) {
          len = src.byteLength;
        }
        let ab =  this.call(this.id.toString(), streamToWrite, "Write", src);
        return this.make_success(ab);
      }

      TempDir() : string {
        let ab =  this.Call("TempDir", new JSON.Str("{}"), "ctx");
        return String.UTF8.decode(ab);
      }

      ReadStream(streamToRead:string, sz:number) : elv_return_type {
        let input = new ArrayBuffer(<i32>sz);
        let ab =  this.call(this.id.toString(), streamToRead, "Read", input);
        return this.make_success(input);
      }

      CloseStream(streamId : string) : elv_return_type{
        let sid = new JSON.Str(QuoteString(streamId));
        return this.Call("CloseStream", sid, "ctx");
      }

      // NewStream creates a new stream and returns its ID.
      NewStream() : string{
        let v = new JSON.Obj;
        let strm = this.Call("NewStream", v, "ctx").getBuffer();
        let streamJson = <JSON.Obj>JSON.parse(String.UTF8.decode(strm));
        let s : JSON.Str | null = streamJson.getString("stream_id")
        if (s == null){
          consoleLog("Failed to get stream_id")
          return "";
        }
        let sid = s.toString();
        this.openStreams.push(sid);
        return sid;
      }

      NewFileStream() : Map<string, string>{
        let m = new Map<string, string>();
        let v = new JSON.Obj;
        let ret = this.Call("NewFileStream", v, "ctx").getJSON();
        let s : JSON.Str | null = ret.getString("stream_id")
        if (s == null){
          consoleLog("Failed to get stream_id")
          return m;
        }
        m.set("stream_id", s.toString());
        this.openStreams.push(s.toString());
        s = ret.getString("file_name");
        if (s == null){
          consoleLog("Failed to get file_name")
          return m;
        }
        m.set("file_name", s.toString());
        return m;
      }
      ProxyHttp(val:JSON.Value) : elv_return_type {
        let ret =  this.Call("ProxyHttp", val, "ext");
        if (ret.isError()){
          return this.make_error("failed to call proxy", ret._1);
        }
        ret._0 = ret.getBuffer();
        return ret;
      }


      FileStreamSize(filename:string) : number {
        consoleLog("FileStreamSize");
        let ert = this.Call("FileStreamSize", JSON.parse("{\"file_name\" : \""+filename+"\"}"), "ctx");
        if (ert.isError()){
          consoleLog("ERROR");
          return -1;
        }
        let jsonRet = String.UTF8.decode(ert.getBuffer());
        consoleLog("FileStream returned="+jsonRet);
        let jret = <JSON.Obj>JSON.parse(jsonRet);

        let jSize = jret.getInteger("file_size");
        if (jSize  == null)
          return -1;
        return Number.parseInt(jSize.toString());
      }

      FileToStream(filename:string, stream:string) : elv_return_type {
        let paramString = `{ "stream_id" : "%STREAM%", "path" : "%PATH%" }`.replace("%STREAM%", stream).replace("%PATH%", filename);
        return this.Call("FileToStream", JSON.parse(paramString), "core");
      }

      Callback(status:i32, content_type:string, sz:i32) : elv_return_type {
        let jResponse = `{"http":{"status": $status, "headers":{"Content-Type":["$content"], "Content-Length":["$len"]}}}`.replace("$status", status.toString()).replace("$content", content_type).replace("$len", sz.toString());
        consoleLog("jResponse="+jResponse);
        let resp = JSON.parse(jResponse);
        return this.Call("Callback", resp, "ctx");
      }


      Call(fnName:string, params:JSON.Value, module:string) : elv_return_type {
        let stringParams = params.toString();
        let jsonString = `{ "jpc" : "1.0", "params" : __PARAMS__, "id" : __ID__, "module" : __MODULE__, "method" : __METHOD__}`;
        jsonString = jsonString.replace("__PARAMS__", stringParams);
        jsonString = jsonString.replace("__MODULE__", QuoteString(module));
        jsonString = jsonString.replace("__ID__", QuoteString(this.id.toString()));
        jsonString = jsonString.replace("__METHOD__", QuoteString(fnName));
        consoleLog("JPC="+jsonString);
        let ab = this.call(this.id.toString(), module, fnName, String.UTF8.encode(jsonString));
        let dec = String.UTF8.decode(ab);
        consoleLog("RETVAL="+dec);
        let jval = JSON.parse(dec);
        if (jval.isObj){
          let j = <JSON.Obj>JSON.parse(dec);
          let vRes : JSON.Value | null = j.get("result");
          if (vRes != null){
              consoleLog("FOUND RESULT");
              return this.make_success(String.UTF8.encode(vRes.toString()));
          }else{
            consoleLog("NO FOUND RESULT");
            let v : JSON.Value | null = j.get("error");
            if (v != null){
              return this.make_error(v.toString(), new Error(v.toString()));
            }
          }
          consoleLog("RETURNING AB");
          return this.make_success(ab);
        }else{
          consoleLog("RETURNING SUCCESS");
          return this.make_success(String.UTF8.encode("SUCCESS"));
        }
      }

      FFMPEGRun(cmdline:string[]):elv_return_type {
        let params = new JSON.Obj;
        let ar = new JSON.Arr();
        for (let i = 0; i < cmdline.length; i++){
          ar.push(JSON.from(cmdline[i]));
        }
        params.set("stream_params", ar);
        return this.Call( "FFMPEGRun", params, "ext");
      }

      QueryParams(j_params:JSON.Obj) : Map<string,string>{
        consoleLog("in QueryParams");
        let m = new Map<string,string>();
        if (j_params.has("http")){
          let j_http = <JSON.Obj>j_params.getObj("http");
          let b = j_http.has("query") ;
          if  (!b){
            consoleLog("QueryParams not present");
            return m;
          }

          let q = j_http.getObj("query");
          if (q == null || q.isNull){
            consoleLog("query params are NULL");
            return m;
          }
          consoleLog("query in q");
          for (let i = 0, k = q.keys.length; i < k; ++i) {
            let key = q.keys[i].toString();
            let valueOrNull : JSON.Value | null = q.get(key);
            consoleLog("key="+key);
            if (valueOrNull != null){
              if (valueOrNull.isArr){
                consoleLog("GOT AN ARRAY!!!");
                let ar =  <JSON.Arr>valueOrNull;
                consoleLog("ELEMENT 0 ="+ar._arr[0].toString());
                m.set(key, ar._arr[0].toString());
              }else{
                m.set(key, valueOrNull.toString());
              }
            }else{
              m.set(key, "VALUE WAS NULL");
            }
          }
        }
        return m;
      }

      QDownloadFile(path:string, hash_or_token:string): elv_return_type{
        consoleLog("QDownloadFile path="+path+" token="+hash_or_token);
        let sid = this.NewStream();
        if (sid == "")
          return this.make_error("Unable to find stream_id", new Error("Bad stream_id"));
        // let streamJson = <JSON.Obj>JSON.parse(String.UTF8.decode(sid));
        // let s : JSON.Str | null = streamJson.getString("stream_id")
        // if (s == null){
        //   return this.make_error("Unable to find stream_id", new Error("Bad stream_id"));
        // }
        // let streamID = s.toString();
        let j = new JSON.Obj();

        j.set<string>("stream_id",sid);
        j.set<string>("path",path);
        j.set<string>("hash_or_token", hash_or_token);
        let ret = this.Call("QFileToStream", j, "core");
        if (ret.isError()){
          return this.make_error("QFileToStream failed", ret._1);
        }
        let jtemp = ret.getJSON().toString();
        consoleLog("json="+jtemp);

        let written:JSON.Value | null = ret.getJSON().get("written");
        if (written == null){
          return this.make_error("QFileToStream returned invalid size", ret._1);
        }
        consoleLog("written="+written.toString());
        let sz = Number.parseInt(written.toString());
        let retStream = this.ReadStream(sid, sz);
        if (retStream.isError()){
           return this.make_error_closer("QFileToStream failed", retStream._1, sid);
        }
        return retStream;

      }

      QUploadToFile(qwt:string, input_data:ArrayBuffer, path:string, mime:string = "") : elv_return_type{
        let sid = this.NewStream();
        let ret_s = this.WriteStream(sid, input_data,input_data.byteLength);
        if (ret_s.isError()){
          return this.make_error("writing part to stream", ret_s.getError());
        }
        let jRet = ret_s.getJSON();
        let written = jRet.getInteger("written");
        let j = new JSON.Obj();

        j.set<string>("qwtoken",qwt);
        j.set<string>("stream_id",sid);
        j.set<string>("path", path);
        j.set<string>("mime", mime);
        j.set<i32>("size", written);
        let method = "QCreateFileFromStream";
        let ret = this.Call(method, j, "core");
        return ret;
      }

      SQMDGetJSON(path:string) : elv_return_type {
        let j = JSON.parse("{\"path\":\""+path+"\"}");
        let method = "SQMDGet";
        return this.Call( method, j, "core");
      }

		JsonCallback(j:JSONEncoder) : elv_return_type {
			//auto jsonCallback = json::parse(args);
			let response = this.call("", "ctx", "Callback", String.UTF8.encode(j.toString()));
      return this.make_success(null);
		}


}
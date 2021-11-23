extern crate wapc;
extern crate base64;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate snailquote;
use wasmtime_provider::WasmtimeEngineProvider;
use elvwasm::ElvError;
use elvwasm::ErrorKinds;
use std::fs::File;
use std::io::BufReader;
use serde_json::json;

use serde::{Deserialize, Serialize};
static mut QFAB: MockFabric = MockFabric{
    fab : None
};

#[derive(Serialize, Deserialize,  Clone, Debug)]
pub struct RootMockFabric {
  pub library:Library
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Object {
  pub hash: String,
  pub id: String,
  pub qlib_id: String,
  #[serde(rename = "type")]
  pub qtype: String,
  pub write_token: String,
  pub meta : serde_json::Map<String, serde_json::Value>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Library {
  pub id: String,
  pub objects: std::vec::Vec<Object>,
}

#[derive(Serialize, Deserialize,  Clone, Debug)]
pub struct MockFabric{
    fab : Option<RootMockFabric>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JPCRequest {
  pub jpc: String,
  pub params: serde_json::Map<String, serde_json::Value>
}

impl MockFabric{
    pub fn init(& mut self, path_to_json:&str) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let file = File::open(path_to_json)?;
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let json_rep:RootMockFabric = serde_json::from_reader(reader)?;
        self.fab = Some(json_rep);
        return Ok("DONE".as_bytes().to_vec())
    }
    pub fn sqmd_get(&self, json_rep:&str) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>{
        println!("in SQMD get");
        let j:JPCRequest = serde_json::from_str(json_rep)?;
        let path = j.params["path"].to_string();
        if  path != ""{
            let fab = self.fab.clone().unwrap();
            let p = &snailquote::unescape(&path).unwrap();
            let mut j_cur = json!(null);
            for k in p.split("/"){
                println!("item k == {}", k);
                if k == ""{
                    continue;
                }
                if j_cur == json!(null) {
                    j_cur = fab.library.objects[0].meta[k].clone();
                }else{
                    j_cur = j_cur[k].clone();
                }
            }
            return Ok(j_cur.to_string().as_bytes().to_vec());
        }else{
            println!("WOOPS 2");
        }
        return Ok("FAILED".as_bytes().to_vec())
    }
    pub fn proxy_http(&self) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>{
        let to_encode = r#"{"url" : {"type" : "application/json"}} "#.as_bytes();
        let enc = base64::encode(to_encode);
        return Ok(format!(r#"{{"result": "{}"}}"#, enc).as_bytes().to_vec())
    }

    pub fn host_callback(i_cb:u64, id:&str, context:&str, method:&str, pkg:&[u8])-> std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>{
        let s_pkg = std::str::from_utf8(pkg)?;
        println!("In host callback, values i_cb = {} id = {} method = {} context = {}, pkg = {}", i_cb, id,method,context, s_pkg);
        match method {
            "SQMDGet" =>{
               unsafe{ QFAB.sqmd_get(s_pkg) }
            }
            "ProxyHttp" => {
                unsafe{ QFAB.proxy_http() }
                }
            _ => {
                Err(ElvError::<String>::new("Method not handled", ErrorKinds::NotExist).into())
            }
        }
    }
}



pub fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("In main");
    if std::env::args().len() < 3 {
        return Err(ElvError::<String>::new("Usage: elvwasm path-to-wasm path-to-fab-json", ErrorKinds::NotExist).into());
    }
    let args: Vec<String> = std::env::args().collect();
    unsafe{QFAB.init(&args[2])?;}
    let module_wat = std::fs::read(&args[1])?;
    let engine = WasmtimeEngineProvider::new(&module_wat, None);
    let host = wapc::WapcHost::new(Box::new(engine), MockFabric::host_callback)?;

    /*
    	"jpc", "1.0",
		"id", id,
		"method", function,
		"qinfo", jcc.QInfo(),
    ID         string         `json:"id"`
    Hash       string         `json:"hash,omitempty"`
    WriteToken string         `json:"write_token,omitempty"`
    QType      string         `json:"type"`
    QLibID     string         `json:"qlib_id,omitempty"`
    Metadata   types.MetaData `json:"meta,omitempty"`
		"params", params,
    */
    host.call("_jpc", r#"{
      "jpc" : "1.0",
      "id" : "id45678933",
      "method" : "proxy",
      "qinfo" : {
          "id" : "id45678934",
          "hash" : "hash44445555",
          "write_token" : "tqw555555",
          "type" : "hash2222222",
          "qlib_id" : "libid6666666"
      },
      "params" : {
        "http": {
          "verb": "unused",
          "path": "/proxy",
          "headers": {
              "Content-type": [ "application/json" ]
            },
            "query": {
                "QUERY": ["fabric"],
                "API_KEY":["AIzaSyCppaD53DdPEetzJugaHc2wW57hG0Y5YWE"],
                "CONTEXT":["012842113009817296384:qjezbmwk0cx"]
            }
          }
        }
      }"#.as_bytes())?;
    Ok(())
}
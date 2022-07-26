use crate::{utils::extract_body, crawler};

use elvwasm::{BitcodeContext, ErrorKinds};
use serde_json::{json, Value};
use std::{error::Error, collections::HashMap};

pub struct Indexer {
    filepath: String,
    fields: Vec<crawler::FieldConfig>
}

impl Indexer {
    pub fn new(bcc: &BitcodeContext, filepath: String, fields: Vec<crawler::FieldConfig>) -> Result<Indexer, Box<dyn Error + Send + Sync>> {
        // Read request
        let http_p = &bcc.request.params.http;
        let query_params = &http_p.query;
        BitcodeContext::log(&format!(
            "In create_index hash={} headers={:#?} query params={:#?}",
            &bcc.request.q_info.hash, &http_p.headers, query_params
        ));
        let _id = &bcc.request.id;

        // Create index in directory
        let mut input_data = json!({
            "directory": "index" //TODO is this correct directory?
        });
        BitcodeContext::log(&format!("before BUILDER"));
        bcc.new_index_builder(input_data)?;
        BitcodeContext::log(&format!("NEW INDEX BUILDER"));

        // Add fields to schema builder
        for field_config in &fields {
            Indexer::add_field_to_schema(bcc, field_config)?;
        }

        // Build index
        input_data = json!({});
        bcc.builder_build(input_data)?;

        return Ok(Indexer { filepath, fields })
    }

    fn add_field_to_schema(bcc: &BitcodeContext, field_config: &crawler::FieldConfig) -> Result<(), Box<dyn Error + Send + Sync>>{ //TODO: Add support for other fields.
        let input_data;
        match field_config.field_type.as_str() {
            "text" => {
                input_data = json!({
                    "name": field_config.name,
                    "type": 1 as u8, //FIXME this should be a TextOption
                    "stored": true,
                });
                let field_title_vec = bcc.builder_add_text_field(input_data)?;
                let ft_json: serde_json::Value = serde_json::from_slice(&field_title_vec)?;
                match extract_body(ft_json.clone()) {
                    Some(o) => o.get("field").unwrap().as_u64(),
                    None => {
                        return Err(Box::new(ErrorKinds::BadHttpParams(
                            "could not find key document-create-id",
                        )))
                    }
                };
                BitcodeContext::log(&format!("ADDED TEXT FIELD."));
            }
            "string" => {
                input_data = json!({
                    "name": field_config.name,
                    "type": 1 as u8, //FIXME this should be a TextOption. What is the right number here?
                    "stored": true,
                });
                let field_title_vec = bcc.builder_add_text_field(input_data)?;
                let ft_json: serde_json::Value = serde_json::from_slice(&field_title_vec)?;
                match extract_body(ft_json.clone()) {
                    Some(o) => o.get("field").unwrap().as_u64(),
                    None => {
                        return Err(Box::new(ErrorKinds::BadHttpParams(
                            "could not find key document-create-id",
                        )))
                    }
                };
                BitcodeContext::log(&format!("ADDED STRING FIELD."));
            }
            _ => panic!("unknown field type"),
        }
        Ok(())
    }
}

struct Writer<'a> {
    bcc: &'a BitcodeContext<'a>,
    fields: HashMap<String, crawler::FieldConfig>
}

impl<'a> Writer<'a> {
    pub fn new(bcc: &'a BitcodeContext, fields: HashMap<String, crawler::FieldConfig>) -> Writer<'a>{
        Writer { bcc, fields }
    }

    pub fn index(&self, uid: &String, data: &Value, fields: &Value) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        assert!(fields.is_object());

        let input = json!({});
        let response_vec = self.bcc.document_create(input)?;
        let response_val: serde_json::Value = serde_json::from_slice(&response_vec)?;
        let doc_id = match extract_body(response_val.clone()) {
            Some(o) => o.get("document-create-id").unwrap().as_u64(),
            None => {
                return self.bcc.make_error_with_kind(ErrorKinds::BadHttpParams(
                    "could not find key document-create-id",
                ))
            }
        }
        .unwrap();
        self.document_add_field("uid", "text", &json!(uid), doc_id)?;
        self.document_add_field("data", "text", data, doc_id)?;

        for field in fields.as_object().unwrap() {
            let field_type = &self.fields.get(field.0).unwrap().field_type;
            self.document_add_field(field.0, field_type, field.1, doc_id)?;
        }
        Ok(Vec::new())
    }

    fn document_add_field(
        &self,
        field_name: &str,
        field_type: &str,
        field_content: &Value,
        doc_id: u64,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match field_type {
            "text" | "string" => {
                let input = json!({
                    "field": field_name,
                    "value": field_content.as_str(),
                    "doc": doc_id
                });
                self.bcc.document_add_text(input)?;
            }
            _ => {
                return Err(Box::new(ErrorKinds::Invalid(
                    "invalid field type encountered",
                )))
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    //use super::*;
    use elvwasm::Request;
    use test_utils::test_metadata::INDEX_CONFIG;
    use std::collections::hash_map::RandomState;
    use crate::{crawler};
    use std::collections::HashMap;
    use serde_json::Value;
    use elvwasm::BitcodeContext;
    use crate::Indexer;

    macro_rules! output_raw_pointers {
        ($raw_ptr:ident, $raw_len:ident) => {
              unsafe { std::str::from_utf8(std::slice::from_raw_parts($raw_ptr, $raw_len)).unwrap_or("unable to convert")}
        }
      }

    #[no_mangle]
    pub extern "C" fn __console_log(ptr: *const u8, len: usize){
      let out_str = output_raw_pointers!(ptr,len);
      println!("console output : {}", out_str);
    }
    #[no_mangle]
    pub extern "C" fn __host_call(
      bd_ptr: *const u8,
      bd_len: usize,
      ns_ptr: *const u8,
      ns_len: usize,
      op_ptr: *const u8,
      op_len: usize,
      ptr: *const u8,
      len: usize,
      ) -> usize {
        let out_bd = output_raw_pointers!(bd_ptr, bd_len);
        let out_ns = output_raw_pointers!(ns_ptr, ns_len);
        let out_op = output_raw_pointers!(op_ptr, op_len);
        let out_ptr = output_raw_pointers!(ptr, len);
        println!("host call bd = {} ns = {} op = {}, ptr={}", out_bd, out_ns, out_op, out_ptr);
        0
    }
    #[no_mangle]
    pub extern "C" fn __host_response(ptr: *const u8){
      println!("host __host_response ptr = {:?}", ptr);
    }

    #[no_mangle]
    pub extern "C" fn __host_response_len() -> usize{
      println!("host __host_response_len");
      0
    }

    #[no_mangle]
    pub extern "C" fn __host_error_len() -> usize{
      println!("host __host_error_len");
      0
    }

    #[no_mangle]
    pub extern "C" fn __host_error(ptr: *const u8){
      println!("host __host_error ptr = {:?}", ptr);
    }

    #[no_mangle]
    pub extern "C" fn __guest_response(ptr: *const u8, len: usize){
      let out_resp = output_raw_pointers!(ptr,len);
      println!("host  __guest_response ptr = {}", out_resp);
    }

    #[no_mangle]
    pub extern "C" fn __guest_error(ptr: *const u8, len: usize){
      let out_error = output_raw_pointers!(ptr,len);
      println!("host  __guest_error ptr = {}", out_error);
    }

    #[no_mangle]
    pub extern "C" fn __guest_request(op_ptr: *const u8, ptr: *const u8){
      println!("host __guest_request op_ptr = {:?} ptr = {:?}", op_ptr, ptr);

    }




    #[test]
    fn test_index() -> () {
        let index_object_meta: Value = serde_json::from_str(INDEX_CONFIG)
            .expect("Could not read index object into json value.");
        let config_value: &Value = &index_object_meta["indexer"]["config"];
        let indexer_config: crawler::IndexerConfig = crawler::IndexerConfig::parse_index_config(config_value)
            .expect("Could not parse indexer config.");
        let new_id = "id123".to_string();
        let req = &Request{
            id: new_id.clone(),
            jpc: "1.0".to_string(),
            method: "foo".to_string(),
            params: elvwasm::JpcParams {
                http: elvwasm::HttpParams {
                    headers: HashMap::<String, Vec<String>, RandomState>::new(),
                    path: "/".to_string(),
                    query: HashMap::<String, Vec<String>, RandomState>::new(),
                    verb: "GET".to_string(),
                    fragment: "".to_string(),
                    content_length: 0,
                    client_ip: "localhost".to_string(),
                    self_url: "localhost".to_string(),
                    proto: "".to_string(),
                    host: "somehost.com".to_string()
                }
            },
            q_info: elvwasm::QInfo { hash: "hqp_123".to_string(), id: new_id, qlib_id: "libfoo".to_string(), qtype: "hq_423234".to_string(), write_token: "tqw_5555".to_string() }
        };
        let bcc = BitcodeContext::new(req);
        //let idx = Indexer::new(&bcc, indexer_config.document.prefix, indexer_config.fields).expect("failed to create index");

    }
}


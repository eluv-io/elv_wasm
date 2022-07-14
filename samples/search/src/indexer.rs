use crate::utils::extract_body;

use elvwasm::{BitcodeContext, ErrorKinds};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{error::Error, collections::HashMap};

#[derive(Deserialize)]
pub struct FieldConfig {
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) field_type: String,
    options: Value,
    pub(crate) paths: Vec<String>,
}

struct Indexer {
    filepath: String,
    fields: HashMap<String, FieldConfig>
}

impl Indexer {
    fn new(bcc: &BitcodeContext, filepath: String, fields: HashMap<String, FieldConfig>) -> Result<Indexer, Box<dyn Error + Send + Sync>> {
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
            Indexer::add_field_to_schema(bcc, field_config.1)?;
        }

        // Build index
        input_data = json!({});
        bcc.builder_build(input_data)?;
        
        return Ok(Indexer { filepath, fields })
    }

    fn add_field_to_schema(bcc: &BitcodeContext, field_config: &FieldConfig) -> Result<(), Box<dyn Error + Send + Sync>>{ //TODO: Add support for other fields.
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
    fields: HashMap<String, FieldConfig>
}

impl<'a> Writer<'a> {
    pub fn new(bcc: &'a BitcodeContext, fields: HashMap<String, FieldConfig>) -> Writer<'a>{
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

extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate thiserror;
extern crate wapc_guest as guest;

use crate::{BitcodeContext};

use serde_json::{json, Value};
use std::str;
use guest::CallResult;

macro_rules! implement_ext_func {
    (
      $(#[$meta:meta])*
      $handler_name:ident,
      $fabric_name:literal
    ) => {
      $(#[$meta])*
      pub fn $handler_name(&'a self, v:Option<serde_json::Value>) -> CallResult {
        let v_call = match v{
          Some(v) => v,
          None => Value::default(),
        };
        let method = $fabric_name;
        let impl_result = self.call_function(method, v_call, "ext")?;
        let id = self.request.id.clone();
        self.make_success_bytes(&impl_result, &id)
      }
    }
  }

impl<'a> BitcodeContext{
    implement_ext_func!(
        /// proxy_http proxies an http request in case of CORS issues
        /// # Arguments
        /// * `v` : a JSON Value
        ///
        /// ```
        /// use serde_json::json;
        ///
        /// fn do_something<'s>(bcc: &'s elvwasm::BitcodeContext) -> wapc_guest::CallResult {
        ///   let v = json!({
        ///         "request_parameters" : {
        ///         "url": "https://www.googleapis.com/customsearch/v1?key=AIzaSyCppaD53DdPEetzJugaHc2wW57hG0Y5YWE&q=fabric&cx=012842113009817296384:qjezbmwk0cx",
        ///         "method": "GET",
        ///         "headers": {
        ///           "Accept": "application/json",
        ///           "Content-Type": "application/json"
        ///         }
        ///      }
        ///   });
        ///   bcc.proxy_http(Some(v))
        /// }
        /// ```
        /// # Returns
        /// * slice of [u8]
        proxy_http,
        "ProxyHttp"
    );

    pub fn new_index_builder(&'a mut self, _v: serde_json::Value) -> CallResult {
        let method = "NewIndexBuilder";
        let vp = json!({});
        let impl_result = self.call_function(method, vp, "ext")?;
        let id = self.request.id.clone();
        self.make_success_bytes(&impl_result, &id)
    }

    pub fn archive_index_to_part(&'a self, dir:&str) -> CallResult {
        self.call_function("ArchiveIndexToPart", json!({"directory" : dir}), "ext")
    }

    pub fn restore_index_from_part(&'a self, content_hash:&str, part_hash:&str) -> CallResult {
        self.call_function("RestoreIndexFromPart", json!({"content-hash" : content_hash, "part-hash": part_hash}), "ext")
    }

    pub fn query_parser_parse_query(&'a self, query:&str) -> CallResult {
        self.call_function("QueryParserParseQuery", json!({"query" : query}), "ext")
    }

    // implement_ext_func!(
    //   /// new_index_builder create a new Tantivy index builder
    //   /// Arguments None
    //   /// ```
    //   /// use serde_json::json;
    //   ///
    //   /// fn do_something<'s>(bcc: &'s elvwasm::BitcodeContext) -> wapc_guest::CallResult {
    //   ///   let v = json!({});
    //   ///   bcc.new_index_builder(v)
    //   /// }
    //   /// ```
    //   new_index_builder, "NewIndexBuilder"
    // );

    implement_ext_func!(
        /// builder_add_text_field adds a new text field to a Tantivy index
        /// # Arguments
        /// * `v` : a JSON Value
        /// ```
        /// use serde_json::json;
        ///
        ///fn do_something<'s>(bcc: &'s elvwasm::BitcodeContext) -> wapc_guest::CallResult {
        ///   let v = json!({
        ///     "name":   "title",
        ///     "type":   1,
        ///     "stored": true,
        ///   });
        ///   bcc.builder_add_text_field(Some(v))
        /// }
        /// ```
        builder_add_text_field,
        "BuilderAddTextField"
    );
    implement_ext_func!(
        /// builder_build builds the new Index
        /// Arguments None
        /// ```
        /// use serde_json::json;
        ///
        ///
        ///fn do_something<'s>(bcc: &'s elvwasm::BitcodeContext) -> wapc_guest::CallResult {
        ///   bcc.builder_build(None)
        /// }
        /// ```
        builder_build,
        "BuilderBuild"
    );

    implement_ext_func!(
        /// builder_create_index create an index from an existing dir
        builder_create_index,
        "BuilderCreateIndex"
    );


    implement_ext_func!(
        /// document_create create a new document for a given Index
        document_create,
        "DocumentCreate"
    );

    implement_ext_func!(
        /// document_add_text add text to a given document
        document_add_text,
        "DocumentAddText"
    );

    implement_ext_func!(
        /// document_create_index creates an index given a set of documents
        document_create_index,
        "DocumentCreateIndex"
    );

    implement_ext_func!(
        /// index_create_writer creates an index writer
        index_create_writer,
        "IndexCreateWriter"
    );

    implement_ext_func!(
        /// index_add_document adds a document to the writer
        index_add_document,
        "IndexWriterAddDocument"
    );

    implement_ext_func!(
        /// index_writer_commit commits the index
        index_writer_commit,
        "IndexWriterCommit"
    );

    implement_ext_func!(
        /// index_reader_builder_create creates a new reader builder on an index
        index_reader_builder_create,
        "IndexReaderBuilderCreate"
    );

    implement_ext_func!(
        /// index_reader_searcher creates a new query parser for the index
        index_reader_searcher,
        "IndexReaderSearcher"
    );

    implement_ext_func!(
        /// reader_builder_query_parser_create creates a ReaderBuilder from a QueryParser
        reader_builder_query_parser_create,
        "ReaderBuilderQueryParserCreate"
    );

    implement_ext_func!(
        /// query_parser_for_index executes ForIndex on the QueryParser
        /// # Arguments
        /// * `v` : a JSON Value
        /// ```
        /// fn do_something<'s>(bcc: &'s elvwasm::BitcodeContext) -> wapc_guest::CallResult {
        ///   let v = serde_json::from_str(r#"{
        ///         "fields" : ["field1", "field2"]
        ///       }
        ///   }"#).unwrap();
        ///   bcc.query_parser_for_index(v)
        /// }
        /// ```
        /// # Returns
        /// * slice of [u8]
        query_parser_for_index,
        "QueryParserForIndex"
    );

    implement_ext_func!(
        /// query_parser_search searches the given QueryParser for the term
        query_parser_search,
        "QueryParserSearch"
    );

    /// ffmpeg_run - runs ffmpeg server side
    /// # Arguments
    /// * `cmdline` - a string array with ffmpeg command line arguments
    /// - note the ffmpeg command line may reference files opened using new_file_stream.
    /// eg
    /// ```
    ///  fn ffmpeg_run_watermark(bcc:&elvwasm::BitcodeContext, height:&str, input_file:&str, new_file:&str, watermark_file:&str, overlay_x:&str, overlay_y:&str) -> wapc_guest::CallResult{
    ///     let base_placement = format!("{}:{}",overlay_x,overlay_y);
    ///     let scale_factor = "[0:v]scale=%SCALE%:-1[bg];[bg][1:v]overlay=%OVERLAY%";
    ///     let scale_factor = &scale_factor.replace("%SCALE%", height).to_string().replace("%OVERLAY%", &base_placement).to_string();
    ///     if input_file == "" || watermark_file == "" || new_file == ""{
    ///       let msg = "parameter validation failed, one file is empty or null";
    ///       return bcc.make_error(msg);
    ///     }
    ///     bcc.ffmpeg_run(["-hide_banner","-nostats","-y","-i", input_file,"-i", watermark_file,"-filter_complex", scale_factor,"-f", "singlejpeg", new_file].to_vec())
    ///  }
    /// ```
    pub fn ffmpeg_run(&'a self, cmdline: Vec<&str>) -> CallResult {
        let params = json!({ "stream_params": cmdline });
        self.call_function("FFMPEGRun", params, "ext")
    }

    pub fn start_bitcode_lro(&'a self, function: &str, args: &serde_json::Value) -> CallResult {
        let params = json!({ "function": function,  "args" : args});
        self.call_function("StartBitcodeLRO", params, "ext")
    }

    pub fn call_external_bitcode(&'a self, function: &str, args: &serde_json::Value, object_hash:&str,code_part_hash:&str) -> CallResult {
        let params = json!({
            "jpc" : "1.0",
            "id" : self.request.id,
            "method" : format!("/{function}"),
            "params" : args,
            "qinfo" : self.request.q_info.clone(),
        });
        let params = json!({ "function": function,  "params" : params, "object_hash" : object_hash, "code_part_hash" : code_part_hash});
        self.call_function("CallExternalBitcode", params, "ext")
    }

}
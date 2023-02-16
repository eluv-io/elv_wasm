extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate thiserror;
extern crate wapc_guest as guest;

use crate::{BitcodeContext, implement_ext_func};

use serde_json::{json, Value};
use std::str;
use guest::CallResult;

impl<'a> BitcodeContext{
    /// new_index_builder Creates a new index builder
    /// storing the resultant blob in a fabric part and returning the part hash in the body of an http response
    ///
    /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/lib.rs#L85)
    ///
    pub fn new_index_builder(&'a mut self, _v: serde_json::Value) -> CallResult {
        let method = "NewIndexBuilder";
        let vp = json!({});
        let impl_result = self.call_function(method, vp, "search")?;
        let id = self.request.id.clone();
        self.make_success_bytes(&impl_result, &id)
    }

    /// archive_index_to_part Uses the index constructed in the fabric directory to form a compressed tar
    /// storing the resultant blob in a fabric part and returning the part hash
    ///
    /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/lib.rs#L121)
    ///
    pub fn archive_index_to_part(&'a self, dir:&str) -> CallResult {
        self.call_function("ArchiveIndexToPart", json!({"directory" : dir}), "search")
    }
    /// restore_index_from_part Extract the part using the supplied part hash restore an archived tantivy index locating the resultant
    /// in a directory on the local node.
    /// # Arguments
    /// * `content_hash` : &str the content object hash
    /// * `part_hash` : &str the part hash returned from [archive_index_to_part]
    ///
    /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/lib.rs#L190)
    ///
    pub fn restore_index_from_part(&'a self, content_hash:&str, part_hash:&str) -> CallResult {
        self.call_function("RestoreIndexFromPart", json!({"content-hash" : content_hash, "part-hash": part_hash}), "search")
    }

    /// query_parser_parse_query Queries the index using the assoicated index query context
    ///
    ///
    /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/searcher.rs#L32)
    ///
    pub fn query_parser_parse_query(&'a self, query:&str) -> CallResult {
        self.call_function("QueryParserParseQuery", json!({"query" : query}), "search")
    }

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
        ///
        /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/lib.rs#L196)
        ///
        builder_add_text_field,
        "BuilderAddTextField",
        "search"
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
        ///
        /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/lib.rs#L201)
        ///
        builder_build,
        "BuilderBuild",
        "search"
    );

    implement_ext_func!(
        /// builder_create_index create an index from an existing dir
        ///
        /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/lib.rs#L202)
        ///
        builder_create_index,
        "BuilderCreateIndex",
        "search"
    );


    implement_ext_func!(
        /// document_create create a new document for a given Index
        ///
        /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/lib.rs#L107)
        ///
        document_create,
        "DocumentCreate",
        "search"
    );

    implement_ext_func!(
        /// document_add_text add text to a given document
        ///
        /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/lib.rs#L115)
        ///
        document_add_text,
        "DocumentAddText",
        "search"
    );

    implement_ext_func!(
        /// document_create_index creates an index given a set of documents
        ///
        /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/lib.rs#L117)
        ///
        document_create_index,
        "DocumentCreateIndex",
        "search"
    );

    implement_ext_func!(
        /// index_create_writer creates an index writer
        ///
        /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/lib.rs#L118)
        ///
        index_create_writer,
        "IndexCreateWriter",
        "search"
    );

    implement_ext_func!(
        /// index_add_document adds a document to the writer
        ///
        /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/lib.rs#L119)
        ///
        index_add_document,
        "IndexWriterAddDocument",
        "search"
    );

    implement_ext_func!(
        /// index_writer_commit commits the index
        ///
        /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/lib.rs#L120)
        ///
        index_writer_commit,
        "IndexWriterCommit",
        "search"
    );

    implement_ext_func!(
        /// index_reader_builder_create creates a new reader builder on an index
        ///
        /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/lib.rs#L203)
        ///
        index_reader_builder_create,
        "IndexReaderBuilderCreate",
        "search"
    );

    implement_ext_func!(
        /// index_reader_searcher creates a new query parser for the index
        ///
        /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/lib.rs#L204)
        ///
        index_reader_searcher,
        "IndexReaderSearcher",
        "search"
    );

    implement_ext_func!(
        /// reader_builder_query_parser_create creates a ReaderBuilder from a QueryParser
        ///
        /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/searcher.rs#L28)
        ///
        reader_builder_query_parser_create,
        "ReaderBuilderQueryParserCreate",
        "search"
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
        ///
        /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/searcher.rs#L30)
        ///
        query_parser_for_index,
        "QueryParserForIndex",
        "search"
    );

    implement_ext_func!(
        /// query_parser_search searches the given QueryParser for the term
        ///
        /// [Example](https://github.com/eluv-io/elv-wasm/blob/d261ece2140e5fc498edc470c6495065d1643b14/samples/search/src/searcher.rs#L34)
        ///
        query_parser_search,
        "QueryParserSearch",
        "search"
    );

}
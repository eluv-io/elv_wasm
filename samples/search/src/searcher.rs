use serde_json::{json};
use elvwasm::{BitcodeContext};
use wapc_guest::CallResult;


pub fn content_query(bcc: &mut BitcodeContext) -> CallResult {
    let searcher = Searcher { bcc: bcc };
    let http_p = &bcc.request.params.http;
    let qp = &http_p.query;
    BitcodeContext::log(&format!("In content_query hash={} headers={:#?} query params={:#?}",&bcc.request.q_info.hash, &http_p.headers, qp));
    searcher.query(qp["query"][0].as_str())?;
    Ok(Vec::new())
}

struct Searcher<'a, 'b> {
    bcc: &'a BitcodeContext<'b>,
}

impl<'a, 'b> Searcher<'a, 'b> {
    

    fn query(&self, query_str: &str) -> CallResult {
        // let hash_part_id_vec = self
        //     .bcc
        //     .sqmd_get_json(&format!("indexer/part/{}", part_name))?;
        // let hash_part_id = serde_json::from_slice(&hash_part_id_vec)?;
        let mut input = json!({});
        self.bcc.index_reader_builder_create(input)?;

        input = json!({});
        self.bcc.reader_builder_query_parser_create(input)?;

        input = serde_json::from_str(r#"{ "fields" : ["title", "body"] } }"#).unwrap();
        self.bcc.query_parser_for_index(input)?;

        input = json!({"query": query_str});
        self.bcc.query_parser_parse_query(input)?;

        input = json!({});
        let _search_results = self.bcc.query_parser_search(input);

        Ok(Vec::new())
    }
}

extern crate elvwasm;
extern crate serde;
extern crate serde_json;
use serde_json::{Map, Value};

use std::collections::HashMap;

use elvwasm::{
    implement_bitcode_module, jpc, make_json_error, make_success_json, register_handler, ErrorKinds,
};
use serde_json::json;

implement_bitcode_module!("get_meta", do_get_meta, "set_meta", do_set_meta);

fn find_path(map: Map<String, Value>, path: String) -> Option<Value> {
    let components: Vec<&str> = path.split('/').filter(|&s| !s.is_empty()).collect();
    let mut current = &map;

    for component in components {
        if let Some(value) = current.get(component) {
            if let Value::Object(obj) = value {
                current = obj;
            } else {
                return Some(value.clone());
            }
        } else {
            return None;
        }
    }

    current.get("").map(|v| v.clone())
}


fn do_get_meta_impl(
    qp: &HashMap<String, Vec<String>>,
    md: Map<String, serde_json::Value>,
) -> CallResult {
    let path = qp.get("path").ok_or(ErrorKinds::BadHttpParams(
        "failed to specify path".to_string(),
    ))?;
    let v = find_path(md, path[0].to_string()).ok_or(ErrorKinds::NotExist(format!(
        "failed to find path {}",
        path[0]
    )))?;
    Ok(serde_json::to_vec(&v)?)
}

#[no_mangle]
fn do_get_meta(bcc: &mut elvwasm::BitcodeContext) -> CallResult {
    let http_p = &bcc.request.params.http;
    let qp = &http_p.query;
    let meta: Map<String, serde_json::Value> =
        serde_json::from_slice(&bcc.sqmd_get_json_resolve(&http_p.path)?)
            .into_iter()
            .collect::<serde_json::Map<String, serde_json::Value>>()
            .into_iter()
            .collect();
    let id = &bcc.request.id;
    let meta_return = do_get_meta_impl(qp, meta)?;
    make_success_json(
        &json!(
        {
            "headers" : "application/json",
            "body" : meta_return,
            "result" : "complete",
        }),
        id,
    )
}

fn do_set_meta_impl(
    _qp: &HashMap<String, Vec<String>>,
    _md: HashMap<String, serde_json::Value>,
) -> CallResult {
    Ok(vec![])
}

#[no_mangle]
fn do_set_meta(bcc: &mut elvwasm::BitcodeContext) -> CallResult {
    let http_p = &bcc.request.params.http;
    let qp = &http_p.query;
    let meta: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&bcc.sqmd_get_json_resolve(&http_p.path)?)
            .into_iter()
            .collect::<serde_json::Map<String, serde_json::Value>>()
            .into_iter()
            .collect();
    let id = &bcc.request.id;
    if bcc.request.q_info.write_token.is_empty() {
        return make_json_error(
            elvwasm::ErrorKinds::NotExist("failed to find valid write token".to_string()),
            id,
        );
    }
    let meta_return = do_set_meta_impl(qp, meta)?;
    make_success_json(
        &json!(
        {
            "headers" : "application/json",
            "body" : meta_return,
            "result" : "complete",
        }),
        id,
    )
}

#[test]
fn test_get_meta_impl() {
    let qp = maplit::hashmap! {
        "path".to_string() => vec!["/meta/test".to_string()],
    };
    let test_json = json!({"meta" : {"test" : "testvalue"}});
    if let Value::Object(obj) = test_json {
        let md: Map<String, Value> = obj;
        let vret = do_get_meta_impl(&qp, md).unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&vret).unwrap(),
            json!("testvalue")
        );
    }
}

#[test]
fn test_set_meta_impl() {
    let qp = HashMap::<String, Vec<String>>::new();
    let md = HashMap::<String, serde_json::Value>::new();
    do_set_meta_impl(&qp, md).unwrap();
}

use petgraph::data::Build;
use petgraph::graph::{Graph, Node, NodeIndex};
use petgraph::visit::{EdgeRef, IntoEdgesDirected};
use petgraph::Directed;
use petgraph::EdgeDirection::{Incoming, Outgoing};
use serde_json::Value;
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};

// FilterDAG represent the automata to filter hierarchical paths
struct FilterDAG {
    pub graph: Graph<(), String>,
    root: NodeIndex,
}

impl FilterDAG {
    pub fn new(fields: &Vec<String>) -> Self {
        // Initialize graph
        let mut graph = Graph::<(), String, Directed>::new();
        let root = graph.add_node(());
        let mut s = Self { graph, root };

        // Build DAG by traversing each field path
        let mut fields_dq = VecDeque::<(VecDeque<&str>, NodeIndex, u8)>::new();
        for field in fields {
            let fields: Vec<&str> = field.split(".").collect(); // Split field path into keys
            let mut el = (VecDeque::from(fields), root, 0);
            fields_dq.push_back(el);
        }
        let mut current_level: i32 = -1;
        let mut seen_keys = HashMap::<(NodeIndex, &str), NodeIndex>::new();
        while fields_dq.len() > 0 {
            let (mut field, parent, level) = fields_dq.pop_front().unwrap();
            if level as i32 > current_level {
                seen_keys.clear();
                current_level = level as i32;
            }
            if field.len() > 0 {
                let key = field.pop_front().unwrap();
                let skey = (parent, key);
                let mut child = s.graph.add_node(());
                if !seen_keys.contains_key(&skey) {
                    let edge = s.graph.add_edge(parent, child, skey.1.to_string());
                    seen_keys.insert(skey, child);
                } else {
                    child = seen_keys[&skey];
                }
                let to_append = (field, child, level + 1);
                fields_dq.push_back(to_append);
            }
        }
        s
    }

    pub fn get_root(&self) -> NodeIndex {
        return self.root;
    }
    // Gets keys for outgoing edges connected to a vertex, given by its NodeIndex
    pub fn next_keys(&self, vertex: NodeIndex) -> Vec<(&String, NodeIndex)> {
        self.graph
            .edges_directed(vertex, Outgoing)
            .map(|edge| (edge.weight(), edge.target()))
            .collect()
    }

    // Returns a tuple of the keys of the path from the root of that vertex
    pub fn prefix(&self, mut vertex: NodeIndex) -> Vec<String> {
        let mut prefix_vec = Vec::new();
        let mut in_edge_option = self.graph.edges_directed(vertex, Incoming).next();
        while let Some(in_edge) = in_edge_option {
            prefix_vec.push(in_edge.weight().to_string());
            vertex = in_edge.source();
            in_edge_option = self.graph.edges_directed(vertex, Incoming).next();
        }
        prefix_vec.reverse();
        return prefix_vec;
    }
}

struct LinkScanner {
    pub filter_dag: FilterDAG,
}

impl LinkScanner {
    pub fn new(filter_dag: FilterDAG) -> LinkScanner {
        return LinkScanner {
            filter_dag: filter_dag,
        };
    }

    fn is_valid_field_type(field_type: &Value) -> bool {
        return field_type.is_string() || field_type.is_i64(); // TODO: check the these are the valid field types.
    }

    pub fn scanner<'a, 'b>(
        &'a mut self,
        dictionary: &'b Value,
        field_state_option: Option<NodeIndex>,
        prefix_option: Option<Vec<String>>,
    ) -> LinkScannerIterator<'a, 'b> {
        let mut scanning_queue: VecDeque<(&'b Value, NodeIndex<u32>, Vec<String>)> =
            VecDeque::<(&Value, NodeIndex, Vec<String>)>::new();
        let field_state = field_state_option.unwrap_or(self.filter_dag.get_root());
        let prefix = prefix_option.unwrap_or(Vec::new());

        scanning_queue.push_back((&dictionary, field_state, prefix));

        return LinkScannerIterator::<'a, 'b> {
            filter_dag: &self.filter_dag,
            scanning_queue,
        };
    }
}

struct LinkScannerIterator<'a, 'b> {
    filter_dag: &'a FilterDAG,
    scanning_queue: VecDeque<(&'b Value, NodeIndex<u32>, Vec<String>)>,
}

impl<'a, 'b> Iterator for LinkScannerIterator<'a, 'b> {
    type Item = (&'b Value, NodeIndex, Vec<String>);

    fn next(&mut self) -> Option<(&'b Value, NodeIndex, Vec<String>)> {
        let mut ret_val: Option<(&'b Value, NodeIndex, Vec<String>)> = None;

        if self.scanning_queue.len() > 0 {
            let (dictionary, field_state, prefix) = self.scanning_queue.pop_front().unwrap();
            if dictionary.is_array() {
                let to_scan = dictionary.as_array().unwrap();
                for (i, value) in to_scan.iter().enumerate() {
                    // Add every submeta to queue to search for links
                    let mut path = prefix.to_vec();
                    path.push(i.to_string());
                    self.scanning_queue.push_back((value, field_state, path));
                }
            } else if dictionary.is_object() {
                if dictionary.get("/").is_some() {
                    // Found link. Set to return value
                    ret_val = Some((dictionary, field_state, prefix.clone()));
                }
                let next_keys = self.filter_dag.next_keys(field_state);
                for (next_key, next_field_state) in next_keys {
                    if next_key.eq("*") {
                        // Add all paths
                        for key in dictionary.as_object().unwrap().keys() {
                            let mut path = prefix.clone();
                            path.push(key.to_string());
                            self.scanning_queue.push_back((
                                dictionary.get(next_key).unwrap(),
                                next_field_state,
                                path,
                            ));
                        }
                    } else if dictionary.get(next_key).is_some() {
                        // Only add path associated with this key
                        let mut path = prefix.clone();
                        path.push(next_key.to_string());
                        self.scanning_queue.push_back((
                            dictionary.get(next_key).unwrap(),
                            next_field_state,
                            path,
                        ));
                    }
                }
            } else if LinkScanner::is_valid_field_type(dictionary) {
                let next_keys = self.filter_dag.next_keys(field_state);
                if next_keys.is_empty() {
                    ret_val = Some((dictionary, field_state, prefix))
                }
            } else {
                panic!("Dict type of {} not supported.", dictionary);
            }
        }
        return ret_val;
    }
}

struct DictScanner {
    pub filter_dag: FilterDAG,
}

impl DictScanner {
    pub fn new(filter_dag: FilterDAG) -> DictScanner {
        return DictScanner {
            filter_dag: filter_dag,
        };
    }

    pub fn scanner<'a, 'b>(
        &'a mut self,
        dictionary: &'b Value,
        field_state_option: Option<NodeIndex>,
        prefix_option: Option<Vec<String>>,
    ) -> DictScannerIterator<'a, 'b> {
        let mut scanning_queue: VecDeque<(&'b Value, NodeIndex<u32>, Vec<String>)> =
            VecDeque::<(&Value, NodeIndex, Vec<String>)>::new();
        let field_state = field_state_option.unwrap_or(self.filter_dag.get_root());
        let prefix = prefix_option.unwrap_or(Vec::new());

        scanning_queue.push_back((&dictionary, field_state, prefix));

        return DictScannerIterator::<'a, 'b> {
            filter_dag: &self.filter_dag,
            scanning_queue,
        };
    }
}

struct DictScannerIterator<'a, 'b> {
    filter_dag: &'a FilterDAG,
    scanning_queue: VecDeque<(&'b Value, NodeIndex<u32>, Vec<String>)>,
}

impl<'a, 'b> Iterator for DictScannerIterator<'a, 'b> {
    type Item = (&'b Value, NodeIndex, Vec<String>);

    fn next(&mut self) -> Option<(&'b Value, NodeIndex, Vec<String>)> {
        if self.scanning_queue.len() > 0 {
            let (dictionary, field_state, prefix) = self.scanning_queue.pop_front().unwrap();
            let next_keys = self.filter_dag.next_keys(field_state);

            if next_keys.is_empty() {
                return Some((dictionary, field_state, prefix));
            } else if dictionary.is_object() {
                for (next_key, next_field_state) in next_keys {
                    if next_key.eq("*") {
                        for key in dictionary.as_object().unwrap().keys() {
                            let mut path = prefix.clone();
                            path.push(key.to_string());
                            self.scanning_queue.push_back((
                                dictionary.get(key).unwrap(),
                                next_field_state,
                                path,
                            ))
                        }
                    } else if dictionary.get(next_key).is_some() {
                        let mut path = prefix.clone();
                        path.push(next_key.to_string());
                        self.scanning_queue.push_back((
                            dictionary.get(next_key).unwrap(),
                            next_field_state,
                            path,
                        ))
                    }
                }
            }
        }
        return None;
    }
}

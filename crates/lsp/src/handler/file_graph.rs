use std::collections::HashMap;

use itertools::Itertools;
use petgraph::graph::NodeIndex;
use petgraph::{Direction, Graph};

#[derive(Default, Debug)]
pub struct FileGraph {
    graph: Graph<String, ()>,
    map: HashMap<String, NodeIndex>,
}

impl FileGraph {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert(&mut self, file: &str, includes: &[&str]) {
        let node = self.insert_node(file.to_string());

        for include in includes {
            let included = self.insert_node(include.to_string());
            self.graph.add_edge(node, included, ());
        }
    }

    pub fn has_references(&self, file: &str) -> bool {
        if let Some(index) = self.get_index(file) {
            self.graph.edges(index).count() > 0
                || self
                    .graph
                    .edges_directed(index, Direction::Incoming)
                    .count()
                    > 0
        } else {
            false
        }
    }

    /// Get all the files that are included in `file`. This will
    /// recurse into files that are included by included files.
    ///
    /// The files are sorted based on the `Ord` impl for `String`
    pub fn get_included_files(&self, file: &str) -> Vec<String> {
        if let Some(index) = self.get_index(file) {
            self.graph
                .neighbors(index)
                .flat_map(|idx| {
                    let file = self.get_file(idx).unwrap();
                    self.get_included_files(file)
                        .into_iter()
                        .chain(std::iter::once(file.to_string()))
                })
                .sorted()
                .dedup()
                .collect_vec()
        } else {
            Default::default()
        }
    }

    /// Gets the files that are related to `file`. This will look at
    /// all files that are included by file and will recurse into
    /// those, it will also look at files that are including `file`.
    ///
    /// The files are sorted based on the `Ord` impl for `String`
    pub fn get_related_files(&self, file: &str) -> Vec<String> {
        if let Some(index) = self.get_index(file) {
            self.recurse_incoming(index)
                .into_iter()
                .chain(self.get_included_files(file).into_iter())
                .sorted()
                .dedup()
                .collect_vec()
        } else {
            Default::default()
        }
    }

    fn recurse_incoming(&self, idx: NodeIndex) -> Vec<String> {
        self.graph
            .neighbors_directed(idx, Direction::Incoming)
            .flat_map(|idx| {
                let file = self.get_file(idx).unwrap();
                self.recurse_incoming(idx)
                    .into_iter()
                    .chain(std::iter::once(file.to_string()))
            })
            .collect_vec()
    }

    /// Attempt to insert a node, if the node already exists then the
    /// existing index will be returned.
    fn insert_node(&mut self, data: String) -> NodeIndex {
        if let Some(index) = self.map.get(&data) {
            *index
        } else {
            let node = self.graph.add_node(data.to_string());
            self.map.insert(data, node);
            node
        }
    }

    /// Get the node index based on the filename.
    fn get_index(&self, file: &str) -> Option<NodeIndex> {
        self.map.get(file).cloned()
    }

    /// Gets the filename from the index.
    fn get_file(&self, idx: NodeIndex) -> Option<&String> {
        self.map.iter().find(|(_, v)| **v == idx).map(|(k, _)| k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_adding_includes() {
        let mut graph = FileGraph::new();
        graph.insert("file2", &["file3"]);
        graph.insert("file1", &["file2", "file4"]);

        let actual = graph.get_included_files("file1");
        assert_eq!(vec!["file2", "file3", "file4"], actual);
    }

    #[test]
    fn test_related_files() {
        let mut graph = FileGraph::new();
        graph.insert("file2", &["file3"]);
        graph.insert("file1", &["file2", "file4"]);

        let actual = graph.get_related_files("file3");
        assert_eq!(vec!["file1", "file2"], actual);
    }

    #[test]
    fn test_related_files_included_by_multiple() {
        let mut graph = FileGraph::new();
        graph.insert("file2", &["file3"]);
        graph.insert("file1", &["file2", "file4", "file3"]);

        let actual = graph.get_related_files("file3");
        assert_eq!(vec!["file1", "file2"], actual);
    }

    #[test]
    fn test_a_b_c() {
        let mut graph = FileGraph::new();
        graph.insert("a", &["b"]);
        graph.insert("b", &["c"]);

        let actual = graph.get_related_files("a");
        assert_eq!(vec!["b", "c"], actual);
        let actual = graph.get_related_files("c");
        assert_eq!(vec!["a", "b"], actual);
        let actual = graph.get_related_files("b");
        assert_eq!(vec!["a", "c"], actual);
    }

    #[test]
    fn test_references() {
        let mut graph = FileGraph::new();
        graph.insert("file2", &["file3"]);
        graph.insert("file1", &["file2", "file4"]);

        assert!(graph.has_references("file1"));
        assert!(graph.has_references("file2"));
        assert!(graph.has_references("file3"));
        assert_eq!(false, graph.has_references("file5"));
    }
}

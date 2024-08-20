use alloc::vec::Vec;

use super::component::{self, ComponentHolder};

#[derive(PartialEq)]
pub(crate) struct NodeDependency {
    pub owner: usize,
    pub consumer: usize,
}

#[derive(Default)]
pub(crate) struct NodeGraph {
    mappings: Vec<NodeDependency>,
}

impl NodeGraph {
    pub(crate) fn insert_node_dependency(&mut self, node_dep: NodeDependency) {
        assert_ne!(node_dep.owner, node_dep.consumer);
        if !self.mappings.contains(&node_dep) {
            self.mappings.push(node_dep);
        }
    }
}

struct NodeMarker {
    pub node_id: usize,
    pub temp_marker: bool,
    pub perm_marker: bool,
}

pub(crate) struct NodeOrderCalc {
    node_graph: NodeGraph,
    node_markers: Vec<NodeMarker>,
}

impl NodeOrderCalc {
    fn build_node_markers(components: &Vec<ComponentHolder>) -> Vec<NodeMarker> {
        let mut node_markers: Vec<NodeMarker> = Vec::new();
        for component in components.iter() {
            node_markers.push(NodeMarker {
                node_id: component.id,
                temp_marker: false,
                perm_marker: false,
            })
        }

        node_markers
    }

    pub(crate) fn new(node_graph: NodeGraph, components: &Vec<ComponentHolder>) -> Self {
        let node_markers = Self::build_node_markers(components);
        Self {
            node_graph,
            node_markers,
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::system::order::{NodeDependency, NodeGraph};

    #[test]
    #[should_panic(expected = "assertion `left != right` failed")]
    fn test_node_graph() {
        let mut node_graph = NodeGraph::default();
        assert_eq!(node_graph.mappings.len(), 0);

        node_graph.insert_node_dependency(NodeDependency {
            owner: 1,
            consumer: 2,
        });
        assert_eq!(node_graph.mappings.len(), 1);

        node_graph.insert_node_dependency(NodeDependency {
            owner: 1,
            consumer: 2,
        });
        assert_eq!(node_graph.mappings.len(), 1);

        node_graph.insert_node_dependency(NodeDependency {
            owner: 1,
            consumer: 1,
        });
    }
}

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

    fn visit_node(&mut self, curr_marker_idx: usize, ordering: &mut Vec<usize>) {
        let curr_marker_id = {
            let curr_marker = self.node_markers.get_mut(curr_marker_idx).unwrap();

            if curr_marker.perm_marker {
                return;
            }
            if curr_marker.temp_marker {
                panic!("Cycle detected in execution order, TODO provide info on how to fix this.")
            }
            curr_marker.temp_marker = true;

            curr_marker.node_id
        };

        let consumers: Vec<usize> = self
            .node_graph
            .mappings
            .iter()
            .filter(|x| x.owner == curr_marker_id)
            .map(|x| x.consumer)
            .collect();
        for consumer_id in consumers {
            let rec_marker_idx = self
                .node_markers
                .iter()
                .enumerate()
                .find(|(_, marker)| consumer_id == marker.node_id)
                .unwrap()
                .0;
            self.visit_node(rec_marker_idx, ordering)
        }

        {
            let curr_marker = self.node_markers.get_mut(curr_marker_idx).unwrap();

            curr_marker.temp_marker = false;
            curr_marker.perm_marker = true;
        };

        ordering.insert(0, curr_marker_id);
    }

    pub(crate) fn calculate_topological_order(&mut self) -> Vec<usize> {
        let mut ordering: Vec<usize> = Vec::new();

        while self.node_markers.iter().any(|x| !x.perm_marker) {
            let select_marker_idx = self
                .node_markers
                .iter()
                .enumerate()
                .find(|(_, marker)| !marker.perm_marker)
                .unwrap()
                .0;
            self.visit_node(select_marker_idx, &mut ordering);
        }

        ordering
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

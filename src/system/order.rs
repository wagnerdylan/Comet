use alloc::vec::Vec;

use super::component::ComponentHolder;

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
    fn build_node_markers(components: &[ComponentHolder]) -> Vec<NodeMarker> {
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

    pub(super) fn new(node_graph: NodeGraph, components: &[ComponentHolder]) -> Self {
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
                panic!("Cycle detected in execution order.")
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
    use alloc::{boxed::Box, vec::Vec};

    use crate::system::{
        component::{Component, ComponentHolder},
        order::{NodeDependency, NodeGraph},
    };

    use super::NodeOrderCalc;

    struct TestComponent();
    impl Component for TestComponent {
        fn dispatch(&mut self, _channel_store: &crate::channel::store::ChannelStore) {}
    }

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

    #[test]
    fn test_build_node_markers() {
        let holders = [
            ComponentHolder {
                component: Box::new(TestComponent {}),
                id: 0,
            },
            ComponentHolder {
                component: Box::new(TestComponent {}),
                id: 1,
            },
            ComponentHolder {
                component: Box::new(TestComponent {}),
                id: 2,
            },
        ];

        let node_markers = NodeOrderCalc::build_node_markers(&holders);

        assert_eq!(node_markers.first().unwrap().node_id, 0);
        assert_eq!(node_markers.get(1).unwrap().node_id, 1);
        assert_eq!(node_markers.last().unwrap().node_id, 2);

        for node_marker in node_markers.iter() {
            assert!(!node_marker.perm_marker);
            assert!(!node_marker.temp_marker);
        }
    }

    #[test]
    fn test_order() {
        let holders = [
            ComponentHolder {
                component: Box::new(TestComponent {}),
                id: 0,
            },
            ComponentHolder {
                component: Box::new(TestComponent {}),
                id: 1,
            },
            ComponentHolder {
                component: Box::new(TestComponent {}),
                id: 2,
            },
        ];

        let mut node_graph = NodeGraph::default();
        assert_eq!(node_graph.mappings.len(), 0);

        node_graph.insert_node_dependency(NodeDependency {
            owner: 1,
            consumer: 2,
        });
        node_graph.insert_node_dependency(NodeDependency {
            owner: 1,
            consumer: 0,
        });
        node_graph.insert_node_dependency(NodeDependency {
            owner: 1,
            consumer: 2,
        });

        let mut order_calc = NodeOrderCalc::new(node_graph, &holders);
        let ordering = order_calc.calculate_topological_order();

        assert_eq!(ordering, Vec::from([1, 2, 0]));
    }

    #[test]
    #[should_panic(expected = "Cycle detected in execution order.")]
    fn test_order_cycle() {
        let holders = [
            ComponentHolder {
                component: Box::new(TestComponent {}),
                id: 0,
            },
            ComponentHolder {
                component: Box::new(TestComponent {}),
                id: 1,
            },
            ComponentHolder {
                component: Box::new(TestComponent {}),
                id: 2,
            },
        ];

        let mut node_graph = NodeGraph::default();
        assert_eq!(node_graph.mappings.len(), 0);

        node_graph.insert_node_dependency(NodeDependency {
            owner: 0,
            consumer: 1,
        });
        node_graph.insert_node_dependency(NodeDependency {
            owner: 1,
            consumer: 2,
        });
        node_graph.insert_node_dependency(NodeDependency {
            owner: 2,
            consumer: 0,
        });

        let mut order_calc = NodeOrderCalc::new(node_graph, &holders);
        // This should fail on dep cycle check.
        let _ordering = order_calc.calculate_topological_order();
    }
}

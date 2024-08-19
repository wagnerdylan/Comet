use alloc::vec::Vec;

use crate::channel::token::ChannelTokenOps;

use super::{
    reg::{Reg, RegMutView, RegReadView},
    token::{ChannelOwnerToken, ChannelReaderToken},
};

struct Channel<'a> {
    pub name: &'a str,
    pub owner_id: usize,
    pub reg: Reg,
}

#[derive(PartialEq)]
struct NodeDependency {
    pub owner: usize,
    pub consumer: usize,
}

#[derive(Default)]
struct NodeGraph {
    pub(super) mappings: Vec<NodeDependency>,
}

impl NodeGraph {
    pub(self) fn insert_node_dependency(&mut self, node_dep: NodeDependency) {
        assert_ne!(node_dep.owner, node_dep.consumer);
        if !self.mappings.contains(&node_dep) {
            self.mappings.push(node_dep);
        }
    }
}

#[derive(Default)]
pub struct ChannelStore<'a> {
    channels: Vec<Channel<'a>>,
    node_graph: NodeGraph,
}

impl<'a> ChannelStore<'a> {
    fn get_existing_channel_idx(&self, name: &'a str) -> Result<usize, ()> {
        for (i, channel) in self.channels.iter().enumerate() {
            if channel.name == name {
                return Ok(i);
            }
        }

        Err(())
    }

    fn is_unique_channel_name(&self, name: &'a str) -> bool {
        let query_result = self.get_existing_channel_idx(name);

        query_result.is_err()
    }

    fn register_channel(&mut self, name: &'a str, owner_id: usize, reg: Reg) -> usize {
        let accessor_id = self.channels.len();
        self.channels.push(Channel {
            name,
            owner_id,
            reg,
        });

        accessor_id
    }

    pub(self) fn register_write_channel(
        &mut self,
        name: &'a str,
        owner_id: usize,
        reg: Reg,
    ) -> ChannelOwnerToken {
        assert!(self.is_unique_channel_name(name));
        assert!(!name.is_empty());
        let accessor_id = self.register_channel(name, owner_id, reg);
        ChannelOwnerToken::new(accessor_id)
    }

    pub(self) fn register_read_channel(
        &mut self,
        name: &'a str,
        read_owner_id: usize,
    ) -> ChannelReaderToken {
        let query_result = self.get_existing_channel_idx(name);
        // TODO handle panic in a better way here.
        let accessor_idx = query_result.unwrap();
        // Associate the consumer (caller) with the owner of the channel for generating the execution ordering of components.
        let channel_owner_id = self.channels.get(accessor_idx).unwrap().owner_id;
        self.node_graph.insert_node_dependency(NodeDependency {
            owner: channel_owner_id,
            consumer: read_owner_id,
        });

        ChannelReaderToken::new(accessor_idx)
    }
}

pub trait RegViewProducer<'a, T, K> {
    fn grab(&'a self, token: &T) -> K;
}

impl<'a> RegViewProducer<'a, ChannelOwnerToken, RegMutView<'a>> for ChannelStore<'a> {
    fn grab(&'a self, token: &ChannelOwnerToken) -> RegMutView<'a> {
        assert!(token.is_valid());
        let accessor_id = token.get_accessor_id();

        if let Some(channel) = self.channels.get(accessor_id) {
            return RegMutView::new(&channel.reg);
        } else {
            panic!("Invalid accessor token.");
        }
    }
}

impl<'a> RegViewProducer<'a, ChannelReaderToken, RegReadView<'a>> for ChannelStore<'a> {
    fn grab(&'a self, token: &ChannelReaderToken) -> RegReadView<'a> {
        assert!(token.is_valid());
        let accessor_id = token.get_accessor_id();

        if let Some(channel) = self.channels.get(accessor_id) {
            return RegReadView::new(&channel.reg);
        } else {
            panic!("Invalid accessor token.");
        }
    }
}

pub struct ChannelWriteBuilder {
    owner_id: usize,
}

impl<'a> ChannelWriteBuilder {
    pub fn new(owner_id: usize) -> ChannelWriteBuilder {
        ChannelWriteBuilder { owner_id }
    }

    pub fn register_write_channel(
        &mut self,
        channel_store: &mut ChannelStore<'a>,
        name: &'a str,
        reg: Reg,
    ) -> ChannelOwnerToken {
        channel_store.register_write_channel(name, self.owner_id, reg)
    }
}

pub struct ChannelReadBuilder {
    owner_id: usize,
}

impl<'a> ChannelReadBuilder {
    pub fn new(owner_id: usize) -> ChannelReadBuilder {
        ChannelReadBuilder { owner_id }
    }

    pub fn register_read_channel(
        &self,
        channel_store: &mut ChannelStore<'a>,
        name: &'a str,
    ) -> ChannelReaderToken {
        channel_store.register_read_channel(name, self.owner_id)
    }
}

#[cfg(test)]
mod unit_tests {
    use alloc::vec::Vec;

    use crate::channel::{
        reg::{Reg, RegGetter},
        token::ChannelTokenOps,
    };

    use super::{ChannelStore, NodeDependency, NodeGraph, RegViewProducer};

    #[test]
    fn test_register_write_channel() {
        let mut channel_store = ChannelStore::default();
        let test1_channel_name = "test1.test.channel";
        let test_owner_id = 1usize;
        let token_test1 =
            channel_store.register_write_channel(test1_channel_name, test_owner_id, Reg::from(8u8));

        assert_eq!(token_test1.get_accessor_id(), 0);

        let test2_channel_name = "test2.test.channel";
        let token_test2 = channel_store.register_write_channel(
            test2_channel_name,
            test_owner_id,
            Reg::from(10u8),
        );

        assert_eq!(token_test2.get_accessor_id(), 1);

        let test1_channel = &channel_store.channels[channel_store
            .get_existing_channel_idx(test1_channel_name)
            .unwrap()];
        assert_eq!(test1_channel.owner_id, 1);
    }

    #[test]
    #[should_panic(expected = "assertion failed: self.is_unique_channel_name(name)")]
    fn test_duplicate_register_write_channel() {
        let mut channel_store = ChannelStore::default();
        let test1_channel_name = "test1.test.channel";
        let test_owner_id = 1usize;
        let token_test1 =
            channel_store.register_write_channel(test1_channel_name, test_owner_id, Reg::from(8u8));

        assert_eq!(token_test1.get_accessor_id(), 0);
        let _token_test2 = channel_store.register_write_channel(
            test1_channel_name,
            test_owner_id,
            Reg::from(10u8),
        );
    }

    #[test]
    #[should_panic(expected = "assertion failed: !name.is_empty(")]
    fn test_empty_name_register_write_channel() {
        let mut channel_store = ChannelStore::default();
        let test1_channel_name = "";
        let test_owner_id = 1usize;
        let _token_test1 =
            channel_store.register_write_channel(test1_channel_name, test_owner_id, Reg::from(8u8));
    }

    #[test]
    fn test_register_read_channel() {
        let mut channel_store = ChannelStore::default();
        let test1_channel_name = "test1.test.channel";
        let test1_owner_id = 1usize;
        channel_store.register_write_channel(test1_channel_name, test1_owner_id, Reg::from(8u8));

        let test2_owner_id = 2usize;
        let test2_read_token =
            channel_store.register_read_channel(test1_channel_name, test2_owner_id);
        let test2_channel_value: u8 = channel_store.grab(&test2_read_token).get();
        assert_eq!(test2_channel_value, 8u8);
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: ()")]
    fn test_empty_register_read_channel() {
        let mut channel_store = ChannelStore::default();
        let test1_channel_name = "test1.test.channel";
        let test_owner_id = 1usize;

        let _test1_read_token =
            channel_store.register_read_channel(test1_channel_name, test_owner_id);
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: ()")]
    fn test_mismatch_register_read_channel() {
        let mut channel_store = ChannelStore::default();
        let test1_channel_name = "test1.test.channel";
        let test_owner_id = 1usize;
        channel_store.register_write_channel(test1_channel_name, test_owner_id, Reg::from(8u8));

        let _test1_read_token =
            channel_store.register_read_channel("test2.test.channel", test_owner_id);
    }

    #[test]
    #[should_panic(expected = "assertion `left != right` failed")]
    fn test_node_graph() {
        let mut node_graph = NodeGraph {
            mappings: Vec::new(),
        };
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

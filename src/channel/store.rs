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
        if !self.mappings.contains(&node_dep) {
            self.mappings.push(node_dep);
        }
    }
}

#[derive(Default)]
pub struct ChannelStore<'a> {
    channels: Vec<Channel<'a>>,
    node_graph: NodeGraph,
    init_complete: bool,
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
        assert!(!self.init_complete);
        assert!(self.is_unique_channel_name(name));
        let accessor_id = self.register_channel(name, owner_id, reg);
        ChannelOwnerToken::new(accessor_id)
    }

    pub(self) fn register_read_channel(
        &mut self,
        name: &'a str,
        read_owner_id: usize,
    ) -> ChannelReaderToken {
        assert!(!self.init_complete);
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
        let accessor_id = token.get_accessor_id();

        if let Some(channel) = self.channels.get(accessor_id) {
            return RegReadView::new(&channel.reg);
        } else {
            panic!("Invalid accessor token.");
        }
    }
}

pub struct ChannelBuilder {
    owner_id: usize,
}

impl<'a> ChannelBuilder {
    pub fn new(owner_id: usize) -> ChannelBuilder {
        ChannelBuilder { owner_id }
    }

    pub fn register_write_channel(
        &mut self,
        channel_store: &mut ChannelStore<'a>,
        name: &'a str,
        reg: Reg,
    ) -> ChannelOwnerToken {
        channel_store.register_write_channel(name, self.owner_id, reg)
    }

    pub fn register_read_channel(
        &self,
        channel_store: &mut ChannelStore<'a>,
        name: &'a str,
    ) -> ChannelReaderToken {
        channel_store.register_read_channel(name, self.owner_id)
    }
}

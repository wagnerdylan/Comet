use alloc::{string::String, vec::Vec};

use crate::{
    channel::token::ChannelTokenOps,
    system::order::{NodeDependency, NodeGraph},
};

use super::{
    reg::{AnyClone, Reg, RegMutView, RegReadView},
    token::{ChannelBehindToken, ChannelOwnerToken, ChannelReaderToken},
};

#[derive(PartialEq, Debug)]
enum IdType {
    Owner(usize),
    // ReaderReq variants are used to track dangling channel readers who create the channel.
    ReaderReq(usize),
}

struct Channel {
    pub name: String,
    pub owner_id: IdType,
    pub reg: Reg,
    pub behind_reg: Option<Reg>,
}

pub struct ChannelStore {
    channels: Vec<Channel>,
    pub(crate) node_graph: Option<NodeGraph>,
    pub(crate) active_behind_channels_idx: Vec<usize>,
}

impl Default for ChannelStore {
    fn default() -> Self {
        Self {
            channels: Vec::default(),
            node_graph: Some(NodeGraph::default()),
            active_behind_channels_idx: Vec::default(),
        }
    }
}

impl ChannelStore {
    fn get_existing_channel_idx(&self, name: &str) -> Result<usize, ()> {
        for (i, channel) in self.channels.iter().enumerate() {
            if channel.name == name {
                return Ok(i);
            }
        }

        Err(())
    }

    fn is_unique_channel_name(&self, name: &str) -> bool {
        let query_result = self.get_existing_channel_idx(name);

        query_result.is_err()
    }

    fn register_channel(&mut self, name: String, owner_id: IdType, reg: Reg) -> usize {
        assert!(self.is_unique_channel_name(name.as_str()));
        assert!(!name.is_empty());
        let accessor_id = self.channels.len();
        self.channels.push(Channel {
            name,
            owner_id,
            reg,
            behind_reg: None,
        });

        accessor_id
    }

    pub(self) fn register_write_channel<T: AnyClone>(
        &mut self,
        name: String,
        owner_id: usize,
        initial_value: T,
    ) -> ChannelOwnerToken<T> {
        let accessor_idx =
            self.register_channel(name, IdType::Owner(owner_id), Reg::new(initial_value));
        ChannelOwnerToken::new(accessor_idx)
    }

    pub(self) fn register_dangling_channel<T: AnyClone>(
        &mut self,
        name: String,
        reader_id: usize,
        initial_value: T,
    ) -> ChannelReaderToken<T> {
        let accessor_idx: usize =
            self.register_channel(name, IdType::ReaderReq(reader_id), Reg::new(initial_value));
        ChannelReaderToken::new(accessor_idx)
    }

    pub(self) fn try_obtain_channel_ownership<T>(
        &mut self,
        name: String,
        owner_id: usize,
    ) -> ChannelOwnerToken<T> {
        let query_result = self.get_existing_channel_idx(name.as_str());
        let accessor_idx =
            query_result.unwrap_or_else(|_| panic!("Channel [{}] does not exist.", name));
        let channel_reader_id = {
            match self.channels.get(accessor_idx).unwrap().owner_id {
                IdType::Owner(_) => panic!("Channel [{}] already has an owner.", name),
                IdType::ReaderReq(id) => id,
            }
        };
        self.channels.get_mut(accessor_idx).unwrap().owner_id = IdType::Owner(owner_id);
        self.node_graph
            .as_mut()
            .unwrap()
            .insert_node_dependency(NodeDependency {
                owner: owner_id,
                consumer: channel_reader_id,
            });
        ChannelOwnerToken::new(accessor_idx)
    }

    pub(self) fn register_read_channel<T>(
        &mut self,
        name: String,
        read_owner_id: usize,
    ) -> ChannelReaderToken<T> {
        let query_result = self.get_existing_channel_idx(name.as_str());
        let accessor_idx =
            query_result.unwrap_or_else(|_| panic!("Channel [{}] does not exist.", name));
        // Associate the consumer (caller) with the owner of the channel for generating the execution ordering of components.
        let channel_owner_id = {
            match self.channels.get(accessor_idx).unwrap().owner_id {
                IdType::Owner(id) => id,
                IdType::ReaderReq(_) => {
                    panic!(
                        "Channel [{}] cannot bind as there is no owner for this channel.",
                        name
                    )
                }
            }
        };
        // Unchecked call to unwrap() is okay here as register calls are only allowed when node_graph is Some().
        self.node_graph
            .as_mut()
            .unwrap()
            .insert_node_dependency(NodeDependency {
                owner: channel_owner_id,
                consumer: read_owner_id,
            });

        ChannelReaderToken::new(accessor_idx)
    }

    pub(self) fn register_read_behind_channel<T>(&mut self, name: String) -> ChannelBehindToken<T> {
        let query_result = self.get_existing_channel_idx(name.as_str());
        let accessor_idx =
            query_result.unwrap_or_else(|_| panic!("Channel [{}] does not exist.", name));

        let channel = self.channels.get_mut(accessor_idx).unwrap();
        if let IdType::ReaderReq(_) = channel.owner_id {
            panic!(
                "Channel [{}] cannot bind as there is no owner for this channel.",
                name
            )
        }

        // Mark the channel as operating as an active behind channel.
        if !self.active_behind_channels_idx.contains(&accessor_idx) {
            self.active_behind_channels_idx.push(accessor_idx);
        }

        // Behind register which is contained by the channel should contain a clone of
        // the initial reg value. This allows behind channel access across all stages of execution.
        channel.behind_reg = Some(channel.reg.clone());

        ChannelBehindToken::new(accessor_idx)
    }

    pub(self) fn query_unowned_dangling_channel_names(&self) -> Vec<String> {
        self.channels
            .iter()
            .filter(|channel| matches!(channel.owner_id, IdType::ReaderReq(_)))
            .map(|channel| channel.name.clone())
            .collect()
    }

    pub(crate) fn update_active_behind_registers(&mut self) {
        for idx in self.active_behind_channels_idx.iter() {
            let channel = self.channels.get_mut(*idx).unwrap();

            match channel.behind_reg.as_mut() {
                Some(reg) => reg.clone_from(&channel.reg),
                None => panic!("Behind register for channel [{}] is None, this register should contain Some() value.", channel.name),
            }
        }
    }
}

pub trait RegViewProducer<'a, C, K> {
    fn grab(&'a self, token: &C) -> K;
}

impl<'a, T: AnyClone + Clone> RegViewProducer<'a, ChannelOwnerToken<T>, RegMutView<'a, T>>
    for ChannelStore
{
    fn grab(&'a self, token: &ChannelOwnerToken<T>) -> RegMutView<'a, T> {
        assert!(token.is_valid());
        let accessor_id = token.get_accessor_id();

        if let Some(channel) = self.channels.get(accessor_id) {
            return RegMutView::new(&channel.reg);
        } else {
            panic!("Invalid accessor token.");
        }
    }
}

impl<'a, T: AnyClone + Clone> RegViewProducer<'a, ChannelReaderToken<T>, RegReadView<'a, T>>
    for ChannelStore
{
    fn grab(&'a self, token: &ChannelReaderToken<T>) -> RegReadView<'a, T> {
        assert!(token.is_valid());
        let accessor_id = token.get_accessor_id();

        if let Some(channel) = self.channels.get(accessor_id) {
            return RegReadView::new(&channel.reg);
        } else {
            panic!("Invalid accessor token.");
        }
    }
}

impl<'a, T: AnyClone + Clone> RegViewProducer<'a, ChannelBehindToken<T>, RegReadView<'a, T>>
    for ChannelStore
{
    fn grab(&'a self, token: &ChannelBehindToken<T>) -> RegReadView<'a, T> {
        assert!(token.is_valid());
        let accessor_id = token.get_accessor_id();

        if let Some(channel) = self.channels.get(accessor_id) {
            return RegReadView::new(channel.behind_reg.as_ref().unwrap());
        } else {
            panic!("Invalid accessor token.");
        }
    }
}

pub struct ChannelWriteBuilder {
    owner_id: usize,
}

impl ChannelWriteBuilder {
    pub fn new(owner_id: usize) -> ChannelWriteBuilder {
        ChannelWriteBuilder { owner_id }
    }

    pub fn register_write_channel<T: AnyClone>(
        &self,
        channel_store: &mut ChannelStore,
        name: String,
        initial_value: T,
    ) -> ChannelOwnerToken<T> {
        channel_store.register_write_channel(name, self.owner_id, initial_value)
    }

    pub fn try_obtain_channel_ownership<T>(
        &self,
        channel_store: &mut ChannelStore,
        name: String,
    ) -> ChannelOwnerToken<T> {
        channel_store.try_obtain_channel_ownership(name, self.owner_id)
    }

    pub fn query_unowned_dangling_channel_names(
        &self,
        channel_store: &ChannelStore,
    ) -> Vec<String> {
        channel_store.query_unowned_dangling_channel_names()
    }
}

pub struct ChannelReadBuilder {
    owner_id: usize,
}

impl ChannelReadBuilder {
    pub fn new(owner_id: usize) -> ChannelReadBuilder {
        ChannelReadBuilder { owner_id }
    }

    pub fn register_read_channel<T>(
        &self,
        channel_store: &mut ChannelStore,
        name: String,
    ) -> ChannelReaderToken<T> {
        channel_store.register_read_channel(name, self.owner_id)
    }

    pub fn register_read_behind_channel<T>(
        &self,
        channel_store: &mut ChannelStore,
        name: String,
    ) -> ChannelBehindToken<T> {
        channel_store.register_read_behind_channel(name)
    }
}

pub struct ChannelDanglingBuilder {
    owner_id: usize,
}

impl ChannelDanglingBuilder {
    pub fn new(owner_id: usize) -> ChannelDanglingBuilder {
        ChannelDanglingBuilder { owner_id }
    }

    pub fn register_dangling_channel<T: AnyClone>(
        &self,
        channel_store: &mut ChannelStore,
        name: String,
        default_value: T,
    ) -> ChannelReaderToken<T> {
        channel_store.register_dangling_channel(name, self.owner_id, default_value)
    }
}

#[cfg(test)]
mod unit_tests {
    use alloc::{string::ToString, vec};

    use crate::channel::{store::IdType, token::ChannelTokenOps};

    use super::{ChannelStore, RegViewProducer};

    #[test]
    fn test_register_write_channel() {
        let mut channel_store = ChannelStore::default();
        let test1_channel_name = "test1.test.channel";
        let test_owner_id = 1usize;
        let token_test1 = channel_store.register_write_channel(
            test1_channel_name.to_string(),
            test_owner_id,
            8u8,
        );

        assert_eq!(token_test1.get_accessor_id(), 0);

        let test2_channel_name = "test2.test.channel";
        let token_test2 = channel_store.register_write_channel(
            test2_channel_name.to_string(),
            test_owner_id,
            10u8,
        );

        assert_eq!(token_test2.get_accessor_id(), 1);

        let test1_channel = &channel_store.channels[channel_store
            .get_existing_channel_idx(test1_channel_name)
            .unwrap()];
        assert_eq!(test1_channel.owner_id, IdType::Owner(1));
    }

    #[test]
    #[should_panic(expected = "assertion failed: self.is_unique_channel_name(name.as_str())")]
    fn test_duplicate_register_write_channel() {
        let mut channel_store = ChannelStore::default();
        let test1_channel_name = "test1.test.channel";
        let test_owner_id = 1usize;
        let token_test1 = channel_store.register_write_channel(
            test1_channel_name.to_string(),
            test_owner_id,
            8u8,
        );

        assert_eq!(token_test1.get_accessor_id(), 0);
        let _token_test2 = channel_store.register_write_channel(
            test1_channel_name.to_string(),
            test_owner_id,
            10u8,
        );
    }

    #[test]
    #[should_panic(expected = "assertion failed: !name.is_empty(")]
    fn test_empty_name_register_write_channel() {
        let mut channel_store = ChannelStore::default();
        let test1_channel_name = "";
        let test_owner_id = 1usize;
        let _token_test1 = channel_store.register_write_channel(
            test1_channel_name.to_string(),
            test_owner_id,
            8u8,
        );
    }

    #[test]
    fn test_register_read_channel() {
        let mut channel_store = ChannelStore::default();
        let test1_channel_name = "test1.test.channel";
        let test1_owner_id = 1usize;
        channel_store.register_write_channel(test1_channel_name.to_string(), test1_owner_id, 8u8);

        let test2_owner_id = 2usize;
        let test2_read_token =
            channel_store.register_read_channel(test1_channel_name.to_string(), test2_owner_id);
        let test2_channel_value: u8 = channel_store.grab(&test2_read_token).get();
        assert_eq!(test2_channel_value, 8u8);
    }

    #[test]
    #[should_panic(expected = "Channel [test1.test.channel] does not exist.")]
    fn test_empty_register_read_channel() {
        let mut channel_store = ChannelStore::default();
        let test1_channel_name = "test1.test.channel";
        let test_owner_id = 1usize;

        let _test1_read_token: crate::channel::token::ChannelReaderToken<usize> =
            channel_store.register_read_channel(test1_channel_name.to_string(), test_owner_id);
    }

    #[test]
    #[should_panic(expected = "Channel [test2.test.channel] does not exist.")]
    fn test_mismatch_register_read_channel() {
        let mut channel_store = ChannelStore::default();
        let test1_channel_name = "test1.test.channel";
        let test_owner_id = 1usize;
        channel_store.register_write_channel(test1_channel_name.to_string(), test_owner_id, 8u8);

        let _test1_read_token: crate::channel::token::ChannelReaderToken<usize> =
            channel_store.register_read_channel("test2.test.channel".to_string(), test_owner_id);
    }

    #[test]
    fn test_dangling_channels() {
        let mut channel_store = ChannelStore::default();
        channel_store.register_dangling_channel("test.test1".to_string(), 1, 90u8);
        channel_store.register_write_channel("test.test2".to_string(), 1, 70u8);
        channel_store.register_dangling_channel("test.test3".to_string(), 1, 90u8);

        assert!(matches!(
            channel_store.channels.first().unwrap().owner_id,
            IdType::ReaderReq(1)
        ));
        assert!(matches!(
            channel_store.channels.get(2).unwrap().owner_id,
            IdType::ReaderReq(1)
        ));
        assert_eq!(
            channel_store.query_unowned_dangling_channel_names(),
            vec!["test.test1".to_string(), "test.test3".to_string()]
        );

        channel_store.try_obtain_channel_ownership::<u8>("test.test1".to_string(), 2);
        assert!(matches!(
            channel_store.channels.first().unwrap().owner_id,
            IdType::Owner(2)
        ));
        assert_eq!(
            channel_store.query_unowned_dangling_channel_names(),
            vec!["test.test3".to_string()]
        );
    }

    #[test]
    #[should_panic(expected = "Channel [test.test1] already has an owner.")]
    fn test_dangling_channels_multi_owner() {
        let mut channel_store = ChannelStore::default();
        channel_store.register_dangling_channel("test.test1".to_string(), 1, 90u8);
        channel_store.try_obtain_channel_ownership::<u8>("test.test1".to_string(), 2);
        channel_store.try_obtain_channel_ownership::<u8>("test.test1".to_string(), 3);
    }

    #[test]
    fn test_behind_channel_register() {
        let mut channel_store = ChannelStore::default();
        channel_store.register_write_channel("test.test1".to_string(), 1, 70u8);
        channel_store.register_read_channel::<u8>("test.test1".to_string(), 2);
        let behind_tok: crate::channel::token::ChannelBehindToken<u8> =
            channel_store.register_read_behind_channel("test.test1".to_string());

        assert_eq!(behind_tok.get_accessor_id(), 0usize);
        assert!(channel_store.channels.first().unwrap().behind_reg.is_some());
        assert_eq!(
            channel_store.active_behind_channels_idx.first().unwrap(),
            &0usize
        );
        assert_eq!(channel_store.active_behind_channels_idx.len(), 1);

        channel_store.register_write_channel("test.test2".to_string(), 1, 70u8);
        assert!(channel_store.channels.get(1).unwrap().behind_reg.is_none());
        assert_eq!(channel_store.active_behind_channels_idx.len(), 1)
    }

    #[test]
    fn test_behind_channel_update() {
        let mut channel_store = ChannelStore::default();
        let write_tok = channel_store.register_write_channel("test.test1".to_string(), 1, 70u8);
        let behind_tok = channel_store.register_read_behind_channel("test.test1".to_string());

        let mut reg_val: u8 = channel_store.grab(&write_tok).get();
        assert_eq!(reg_val, 70u8);
        let mut reg_behind_val: u8 = channel_store.grab(&behind_tok).get();
        assert_eq!(reg_behind_val, 70u8);

        channel_store.grab(&write_tok).set(100u8);
        reg_val = channel_store.grab(&write_tok).get();
        assert_eq!(reg_val, 100u8);
        reg_behind_val = channel_store.grab(&behind_tok).get();
        assert_eq!(reg_behind_val, 70u8);

        channel_store.update_active_behind_registers();

        channel_store.grab(&write_tok).set(100u8);
        reg_val = channel_store.grab(&write_tok).get();
        assert_eq!(reg_val, 100u8);
        reg_behind_val = channel_store.grab(&behind_tok).get();
        assert_eq!(reg_behind_val, 100u8);
    }
}

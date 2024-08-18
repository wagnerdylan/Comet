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

pub struct ChannelStore<'a, const N: usize> {
    channels: [Option<Channel<'a>>; N],
    current_size: usize,
    init_complete: bool,
}

impl<'a, const N: usize> ChannelStore<'a, N> {
    fn get_existing_channel_accessor_id(&self, name: &'a str) -> Result<usize, ()> {
        for (i, channel_o) in self.channels.iter().enumerate() {
            if let Some(some_channel) = channel_o {
                if some_channel.name == name {
                    return Ok(i);
                }
            }
        }

        Err(())
    }

    fn is_unique_channel_name(&self, name: &'a str) -> bool {
        let query_result = self.get_existing_channel_accessor_id(name);

        query_result.is_err()
    }

    fn register_channel(&mut self, name: &'a str, owner_id: usize, reg: Reg) -> usize {
        assert!(self.current_size < self.channels.len());

        let accessor_id = self.current_size;
        self.channels[self.current_size] = Some(Channel {
            name,
            owner_id,
            reg,
        });
        self.current_size += 1;

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

    pub(self) fn register_read_channel(&self, name: &'a str) -> ChannelReaderToken {
        assert!(!self.init_complete);
        let query_result = self.get_existing_channel_accessor_id(name);
        // TODO handle panic in a better way here.
        let accessor_id = query_result.unwrap();
        ChannelReaderToken::new(accessor_id)
    }
}

impl<'a, const N: usize> Default for ChannelStore<'a, N> {
    fn default() -> Self {
        Self {
            channels: [const { None }; N],
            current_size: 0,
            init_complete: false,
        }
    }
}

pub trait RegViewProducer<'a, T, K> {
    fn grab(&'a self, token: &T) -> K;
}

impl<'a, const N: usize> RegViewProducer<'a, ChannelOwnerToken, RegMutView<'a>>
    for ChannelStore<'a, N>
{
    fn grab(&'a self, token: &ChannelOwnerToken) -> RegMutView<'a> {
        let accessor_id = token.get_accessor_id();
        assert!(accessor_id < self.current_size);

        if let Some(Some(channel)) = self.channels.get(accessor_id) {
            return RegMutView::new(&channel.reg);
        } else {
            panic!("Invalid accessor token.");
        }
    }
}

impl<'a, const N: usize> RegViewProducer<'a, ChannelReaderToken, RegReadView<'a>>
    for ChannelStore<'a, N>
{
    fn grab(&'a self, token: &ChannelReaderToken) -> RegReadView<'a> {
        let accessor_id = token.get_accessor_id();
        assert!(accessor_id < self.current_size);

        if let Some(Some(channel)) = self.channels.get(accessor_id) {
            return RegReadView::new(&channel.reg);
        } else {
            panic!("Invalid accessor token.");
        }
    }
}

pub struct ChannelBuilder<const N: usize> {
    owner_id: usize,
}

impl<'a, const N: usize> ChannelBuilder<N> {
    pub(super) fn new(
        channel_store: &mut ChannelStore<'a, N>,
        owner_id: usize,
    ) -> ChannelBuilder<N> {
        ChannelBuilder { owner_id }
    }

    pub fn register_write_channel(
        &mut self,
        channel_store: &mut ChannelStore<'a, N>,
        name: &'a str,
        reg: Reg,
    ) -> ChannelOwnerToken {
        channel_store.register_write_channel(name, self.owner_id, reg)
    }

    pub fn register_read_channel(
        &self,
        channel_store: &mut ChannelStore<'a, N>,
        name: &'a str,
    ) -> ChannelReaderToken {
        channel_store.register_read_channel(name)
    }
}

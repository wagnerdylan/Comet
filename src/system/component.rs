use alloc::boxed::Box;

use crate::channel::store::{
    ChannelDanglingBuilder, ChannelReadBuilder, ChannelStore, ChannelWriteBuilder,
};

pub trait Component {
    fn register_dangling_channels(
        &mut self,
        _channel_builder: ChannelDanglingBuilder,
        _channel_store: &mut ChannelStore,
    ) {
    }

    fn register_write_channels(
        &mut self,
        _channel_builder: ChannelWriteBuilder,
        _channel_store: &mut ChannelStore,
    ) {
    }
    fn register_read_channels(
        &mut self,
        _channel_builder: ChannelReadBuilder,
        _channel_store: &mut ChannelStore,
    ) {
    }
    fn dispatch(&mut self, channel_store: &ChannelStore);
}

pub(super) struct ComponentHolder {
    pub component: Box<dyn Component>,
    pub id: usize,
}

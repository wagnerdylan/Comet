use alloc::boxed::Box;

use crate::channel::store::{ChannelReadBuilder, ChannelStore, ChannelWriteBuilder};

pub trait Component {
    fn register_write_channels(
        &mut self,
        channel_builder: ChannelWriteBuilder,
        channel_store: &mut ChannelStore,
    ) {
    }
    fn register_read_channels(
        &mut self,
        channel_builder: ChannelReadBuilder,
        channel_store: &mut ChannelStore,
    ) {
    }
    fn dispatch(&mut self, channel_store: &ChannelStore);
}

pub(super) struct ComponentHolder {
    pub component: Box<dyn Component>,
    pub id: usize,
}

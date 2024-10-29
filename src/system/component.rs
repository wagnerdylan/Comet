use alloc::boxed::Box;

use crate::channel::store::{
    ChannelDanglingBuilder, ChannelReadBuilder, ChannelStore, ChannelWriteBuilder,
};

pub trait Component {
    /// Dangling channel registration for a given component is done within this method.
    /// This method is called first by the Runner API as dangling channel ownership may be
    /// "picked up" by other components.
    fn register_dangling_channels(
        &mut self,
        _channel_builder: ChannelDanglingBuilder,
        _channel_store: &mut ChannelStore,
    ) {
    }

    /// Write or "owned" channels are registered within this method. This method is called
    /// second after "register_dangling_channels" as dangling channels may be picked up for
    /// ownership here.
    fn register_write_channels(
        &mut self,
        _channel_builder: ChannelWriteBuilder,
        _channel_store: &mut ChannelStore,
    ) {
    }

    /// Read channels are registered within this channel including channels which are
    /// updated on the previous dispatch (behind channels). Multiple readers may bind to
    /// a single write or owned channel.
    fn register_read_channels(
        &mut self,
        _channel_builder: ChannelReadBuilder,
        _channel_store: &mut ChannelStore,
    ) {
    }

    /// Runtime code is called within this method for execution per runner dispatch.
    fn dispatch(&mut self, channel_store: &ChannelStore);
}

pub(super) struct ComponentHolder {
    pub component: Box<dyn Component>,
    /// The "id" field is used to track owners and consumers of channels for layout generation of execution topology.
    pub id: usize,
}

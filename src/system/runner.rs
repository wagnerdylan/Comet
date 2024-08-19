use alloc::{boxed::Box, vec::Vec};

use crate::channel::store::{ChannelReadBuilder, ChannelStore, ChannelWriteBuilder};

use super::component::{Component, ComponentHolder};

#[derive(Default)]
pub struct Runner<'a> {
    components: Vec<ComponentHolder>,
    channel_store: ChannelStore<'a>,
    component_counter: usize,
    init_complete: bool,
}

impl<'a> Runner<'a> {
    pub fn add_component(&mut self, component: Box<dyn Component>) {
        assert!(!self.init_complete);
        self.components.push(ComponentHolder {
            component,
            id: self.component_counter,
        });
        self.component_counter += 1;
    }

    pub fn initialize(&mut self) {
        assert!(!self.init_complete);
        // Register component write channels with the channel store. Write channels must be registered before
        // read channels.
        for component_holder in self.components.iter_mut() {
            let write_builder = ChannelWriteBuilder::new(component_holder.id);
            component_holder
                .component
                .register_write_channels(write_builder, &mut self.channel_store);
        }

        // Register component read channels with the channel store. Read channels are registered to existing channels,
        // as such, read channels are registered after write channels.
        for component_holder in self.components.iter_mut() {
            let read_builder = ChannelReadBuilder::new(component_holder.id);
            component_holder
                .component
                .register_read_channels(read_builder, &mut self.channel_store);
        }

        // TODO set execution order.

        self.init_complete = true;
    }

    pub fn dispatch_components(&mut self) {
        assert!(self.init_complete);

        for component_holder in self.components.iter_mut() {
            component_holder.component.dispatch(&self.channel_store);
        }
    }
}

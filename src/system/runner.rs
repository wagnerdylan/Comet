use alloc::{boxed::Box, vec::Vec};

use crate::{
    channel::store::{ChannelReadBuilder, ChannelStore, ChannelWriteBuilder},
    system::order::NodeOrderCalc,
};

use super::component::{Component, ComponentHolder};

#[derive(Default)]
pub struct Runner {
    components: Vec<ComponentHolder>,
    channel_store: ChannelStore,
    component_counter: usize,
    init_complete: bool,
}

impl Runner {
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

        // Calculate and modify execution order of the inserted components to create an
        // execution topological sequence.
        let mut node_order_data = NodeOrderCalc::new(
            self.channel_store.node_graph.take().unwrap(),
            &self.components,
        );
        let ordering = node_order_data.calculate_topological_order();
        self.modify_component_ordering(ordering);

        self.init_complete = true;
    }

    pub fn dispatch_components(&mut self) {
        assert!(self.init_complete);

        for component_holder in self.components.iter_mut() {
            component_holder.component.dispatch(&self.channel_store);
        }
    }

    fn modify_component_ordering(&mut self, ordering: Vec<usize>) {
        for (insert_idx, component_id) in ordering.iter().enumerate() {
            let component_idx = self
                .components
                .iter()
                .enumerate()
                .find(|(_, comp_holder)| comp_holder.id == *component_id)
                .unwrap()
                .0;
            let component = self.components.remove(component_idx);
            self.components.insert(insert_idx, component);
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use alloc::{boxed::Box, vec::Vec};

    use crate::system::component::Component;

    use super::Runner;

    struct TestComponent();
    impl Component for TestComponent {
        fn dispatch(&mut self, _channel_store: &crate::channel::store::ChannelStore) {}
    }

    #[test]
    fn test_runner_insertion() {
        let mut runner = Runner::default();
        runner.add_component(Box::new(TestComponent()));
        runner.add_component(Box::new(TestComponent()));
        runner.add_component(Box::new(TestComponent()));
        runner.add_component(Box::new(TestComponent()));

        assert_eq!(runner.components.len(), 4);
        assert_eq!(runner.components.first().unwrap().id, 0);
        assert_eq!(runner.components.get(1).unwrap().id, 1);
        assert_eq!(runner.components.get(2).unwrap().id, 2);
        assert_eq!(runner.components.last().unwrap().id, 3);
    }

    #[test]
    #[should_panic(expected = "assertion failed: self.init_complete")]
    fn test_init_not_complete() {
        let mut runner = Runner::default();
        runner.dispatch_components();
    }

    #[test]
    fn test_modify_component_ordering() {
        let mut runner = Runner::default();
        runner.add_component(Box::new(TestComponent()));
        runner.add_component(Box::new(TestComponent()));
        runner.add_component(Box::new(TestComponent()));
        runner.add_component(Box::new(TestComponent()));

        runner.modify_component_ordering(Vec::from([3usize, 2, 1, 0]));

        assert_eq!(runner.components.len(), 4);
        assert_eq!(runner.components.first().unwrap().id, 3);
        assert_eq!(runner.components.get(1).unwrap().id, 2);
        assert_eq!(runner.components.get(2).unwrap().id, 1);
        assert_eq!(runner.components.last().unwrap().id, 0);
    }

    #[test]
    fn test_init() {
        let mut runner = Runner::default();
        runner.add_component(Box::new(TestComponent()));
        runner.add_component(Box::new(TestComponent()));
        runner.add_component(Box::new(TestComponent()));
        runner.add_component(Box::new(TestComponent()));
    }
}

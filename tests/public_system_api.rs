use comet::{
    channel::{
        reg::{Reg, RegGetter, RegSetter},
        store::RegViewProducer,
        token::{ChannelOwnerToken, ChannelReaderToken},
    },
    system::{component::Component, runner::Runner},
};

extern crate comet;

struct TestProducer {
    pub channel_tok: ChannelOwnerToken,
    pub channel_name: &'static str,
    pub channel_value: i64,
}

impl Component for TestProducer {
    fn register_write_channels(
        &mut self,
        mut channel_builder: comet::channel::store::ChannelWriteBuilder,
        channel_store: &mut comet::channel::store::ChannelStore,
    ) {
        self.channel_tok = channel_builder.register_write_channel(
            channel_store,
            self.channel_name,
            Reg::from(self.channel_value),
        );
    }

    fn dispatch(&mut self, channel_store: &comet::channel::store::ChannelStore) {
        channel_store
            .grab(&self.channel_tok)
            .set(self.channel_value);
    }
}

struct TestAdder {
    pub input_channel_tok: ChannelReaderToken,
    pub input_channel_name: &'static str,
    pub output_channel_tok: ChannelOwnerToken,
    pub output_channel_name: &'static str,
}

impl Component for TestAdder {
    fn register_write_channels(
        &mut self,
        mut channel_builder: comet::channel::store::ChannelWriteBuilder,
        channel_store: &mut comet::channel::store::ChannelStore,
    ) {
        self.output_channel_tok = channel_builder.register_write_channel(
            channel_store,
            self.output_channel_name,
            Reg::from(0i64),
        )
    }

    fn register_read_channels(
        &mut self,
        channel_builder: comet::channel::store::ChannelReadBuilder,
        channel_store: &mut comet::channel::store::ChannelStore,
    ) {
        self.input_channel_tok =
            channel_builder.register_read_channel(channel_store, self.input_channel_name);
    }

    fn dispatch(&mut self, channel_store: &comet::channel::store::ChannelStore) {
        let input_value: i64 = channel_store.grab(&self.input_channel_tok).get();
        let current_count: i64 = channel_store.grab(&self.output_channel_tok).get();
        channel_store
            .grab(&self.output_channel_tok)
            .set(current_count + input_value);
    }
}

#[test]
fn runnner_api() {
    let producer_42 = TestProducer {
        channel_tok: ChannelOwnerToken::default(),
        channel_name: "test.channel",
        channel_value: 42,
    };
    let adder = TestAdder {
        input_channel_tok: ChannelReaderToken::default(),
        input_channel_name: "test.channel",
        output_channel_tok: ChannelOwnerToken::default(),
        output_channel_name: "test.channel.add",
    };

    let mut runner = Runner::default();

    runner.add_component(Box::new(adder));
    runner.add_component(Box::new(producer_42));

    runner.initialize();

    runner.dispatch_components();
}

use comet::{
    channel::{
        reg::Reg,
        store::RegViewProducer,
        token::{ChannelBehindToken, ChannelOwnerToken, ChannelReaderToken},
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
            self.channel_name.to_string(),
            Reg::new(self.channel_value),
        );
    }

    fn dispatch(&mut self, channel_store: &comet::channel::store::ChannelStore) {
        channel_store
            .grab(&self.channel_tok)
            .set(self.channel_value);
    }
}

struct TestModifier {
    pub channel_tok: ChannelOwnerToken,
}

impl Component for TestModifier {
    fn register_write_channels(
        &mut self,
        mut channel_builder: comet::channel::store::ChannelWriteBuilder,
        channel_store: &mut comet::channel::store::ChannelStore,
    ) {
        let mut dangle_names = channel_builder.query_unowned_dangling_channel_names(channel_store);
        self.channel_tok = channel_builder
            .try_obtain_channel_ownership(channel_store, dangle_names.pop().unwrap());
    }

    fn dispatch(&mut self, channel_store: &comet::channel::store::ChannelStore) {
        let value: i64 = channel_store.grab(&self.channel_tok).get();
        channel_store.grab(&self.channel_tok).set(value + 1);
    }
}

struct TestAdder {
    pub input_channel_tok: ChannelReaderToken,
    pub input_channel_name: &'static str,
    pub output_channel_tok: ChannelOwnerToken,
    pub output_channel_name: &'static str,
    pub mod_channel_tok: ChannelReaderToken,
    pub mod_channel_name: &'static str,
    call_count: usize,
}

impl Component for TestAdder {
    fn register_dangling_channels(
        &mut self,
        channel_builder: comet::channel::store::ChannelDanglingBuilder,
        channel_store: &mut comet::channel::store::ChannelStore,
    ) {
        self.mod_channel_tok = channel_builder.register_dangling_channel(
            channel_store,
            self.mod_channel_name.to_string(),
            Reg::new(10i64),
        );
    }

    fn register_write_channels(
        &mut self,
        mut channel_builder: comet::channel::store::ChannelWriteBuilder,
        channel_store: &mut comet::channel::store::ChannelStore,
    ) {
        self.output_channel_tok = channel_builder.register_write_channel(
            channel_store,
            self.output_channel_name.to_string(),
            Reg::new(0i64),
        )
    }

    fn register_read_channels(
        &mut self,
        channel_builder: comet::channel::store::ChannelReadBuilder,
        channel_store: &mut comet::channel::store::ChannelStore,
    ) {
        self.input_channel_tok = channel_builder
            .register_read_channel(channel_store, self.input_channel_name.to_string());
    }

    fn dispatch(&mut self, channel_store: &comet::channel::store::ChannelStore) {
        let input_value: i64 = channel_store.grab(&self.input_channel_tok).get();
        let current_count: i64 = channel_store.grab(&self.output_channel_tok).get();
        let mod_value: i64 = channel_store.grab(&self.mod_channel_tok).get();
        channel_store
            .grab(&self.output_channel_tok)
            .set(current_count + input_value + mod_value);

        let assert_values = [51i64, 103];
        assert_eq!(
            assert_values[self.call_count].clone(),
            channel_store.grab(&self.output_channel_tok).get::<i64>()
        );

        self.call_count += 1;
    }
}

struct TestCycleRW {
    read_name: &'static str,
    read_tok: Option<ChannelReaderToken>,
    behind_tok: Option<ChannelBehindToken>,
    as_behind: bool,
    write_name: &'static str,
    write_tok: ChannelOwnerToken,
    call_count: usize,
}

impl Component for TestCycleRW {
    fn register_write_channels(
        &mut self,
        mut channel_builder: comet::channel::store::ChannelWriteBuilder,
        channel_store: &mut comet::channel::store::ChannelStore,
    ) {
        self.write_tok = channel_builder.register_write_channel(
            channel_store,
            self.write_name.to_string(),
            Reg::new(34i64),
        );
    }

    fn register_read_channels(
        &mut self,
        channel_builder: comet::channel::store::ChannelReadBuilder,
        channel_store: &mut comet::channel::store::ChannelStore,
    ) {
        if self.as_behind {
            self.behind_tok = Some(
                channel_builder
                    .register_read_behind_channel(channel_store, self.read_name.to_string()),
            );
        } else {
            self.read_tok = Some(
                channel_builder.register_read_channel(channel_store, self.read_name.to_string()),
            );
        }
    }

    fn dispatch(&mut self, channel_store: &comet::channel::store::ChannelStore) {
        let assert_values = [34i64, 100];
        if self.as_behind {
            assert_eq!(
                channel_store
                    .grab(self.behind_tok.as_ref().unwrap())
                    .get::<i64>(),
                assert_values[self.call_count]
            );
        } else {
            assert_eq!(
                channel_store
                    .grab(self.read_tok.as_ref().unwrap())
                    .get::<i64>(),
                100i64
            );
        }
        channel_store.grab(&self.write_tok).set(100i64);

        self.call_count += 1;
    }
}

#[test]
fn runner_api() {
    let producer_42 = TestProducer {
        channel_tok: ChannelOwnerToken::default(),
        channel_name: "test.channel",
        channel_value: 40,
    };
    let adder = TestAdder {
        input_channel_tok: ChannelReaderToken::default(),
        input_channel_name: "test.channel",
        output_channel_tok: ChannelOwnerToken::default(),
        output_channel_name: "test.channel.add",
        mod_channel_tok: ChannelReaderToken::default(),
        mod_channel_name: "test.channel.mod",
        call_count: 0,
    };
    let modifier = TestModifier {
        channel_tok: ChannelOwnerToken::default(),
    };
    let cycle_1 = TestCycleRW {
        read_name: "test.channel.cycle.2",
        as_behind: false,
        write_name: "test.channel.cycle.1",
        read_tok: Some(ChannelReaderToken::default()),
        behind_tok: None,
        write_tok: ChannelOwnerToken::default(),
        call_count: 0,
    };
    let cycle_2 = TestCycleRW {
        read_name: "test.channel.cycle.1",
        as_behind: true,
        write_name: "test.channel.cycle.2",
        read_tok: None,
        behind_tok: Some(ChannelBehindToken::default()),
        write_tok: ChannelOwnerToken::default(),
        call_count: 0,
    };

    let mut runner = Runner::default();

    runner.add_component(Box::new(adder));
    runner.add_component(Box::new(producer_42));
    runner.add_component(Box::new(modifier));

    runner.add_component(Box::new(cycle_1));
    runner.add_component(Box::new(cycle_2));

    runner.initialize();

    runner.dispatch_components();
    runner.dispatch_components();
}

use comet::channel::{
    reg::Reg,
    store::{ChannelReadBuilder, ChannelStore, ChannelWriteBuilder, RegViewProducer},
};

extern crate comet;

#[test]
fn channel_api() {
    let mut channel_store = ChannelStore::default();
    let channel_builder = ChannelWriteBuilder::new(0usize);

    let t_1_o = channel_builder.register_write_channel(
        &mut channel_store,
        "test1.test.channel".to_string(),
        Reg::new(42u32),
    );
    let read_channel_builder = ChannelReadBuilder::new(1usize);
    let t_1_r = read_channel_builder
        .register_read_channel(&mut channel_store, "test1.test.channel".to_string());
    let _t_2_o = channel_builder.register_write_channel(
        &mut channel_store,
        "test2.test.channel".to_string(),
        Reg::new(9000.0f64),
    );

    let mut_reg = channel_store.grab(&t_1_o);
    let mut_reg_val: u32 = mut_reg.get();
    assert_eq!(mut_reg_val, 42u32);

    mut_reg.set(10u32);
    let new_mut_reg_val: u32 = mut_reg.get();
    assert_eq!(new_mut_reg_val, 10u32);

    let read_reg = channel_store.grab(&t_1_r);
    let reg_val: u32 = read_reg.get();
    assert_eq!(reg_val, 10u32);
}

use comet::channel::{
    reg::{Reg, RegGetter, RegMutView, RegReadView, RegSetter},
    store::{ChannelBuilder, ChannelStore, RegViewProducer},
};

extern crate comet;

#[test]
fn register_api() {
    let test_from: &Reg = &Reg::from(true);
    test_from.set(false);
    let test_value: bool = test_from.get();

    assert!(!test_value);

    {
        let read_view = RegReadView::new(test_from);
        let read_test_value: bool = read_view.get();
        assert!(!read_test_value);
    }
    {
        let write_view = RegMutView::new(test_from);
        write_view.set(true);
        let write_test_value: bool = write_view.get();
        assert!(write_test_value);
    }
}

/*
#[test]
fn channel_api() {
    let mut channel_store: ChannelStore<10> = ChannelStore::default();
    let mut channel_builder = ChannelBuilder::new(&mut channel_store);

    let t_1_o = channel_builder.register_write_channel(
        &mut channel_store,
        "test1.test.channel",
        Reg::from(42u32),
    );
    let t_1_r = channel_builder.register_read_channel(&mut channel_store, "test1.test.channel");
    let _t_2_o = channel_builder.register_write_channel(
        &mut channel_store,
        "test2.test.channel",
        Reg::from(9000.0f64),
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
*/

use comet::channel::{
    reg::{Reg, RegGetter, RegMutView, RegReadView, RegSetter},
    store::{ChannelBuilder, ChannelStore, RegViewProducer},
};

extern crate comet;

#[test]
fn register_api() {
    let test_from: &Reg = &Reg::from(1u64);
    test_from.set(30u64);
    let test_value: u64 = test_from.get();

    assert_eq!(test_value, 30);

    {
        let read_view = RegReadView::new(test_from);
        let read_test_value: u64 = read_view.get();
        assert_eq!(read_test_value, 30);
    }
    {
        let write_view = RegMutView::new(test_from);
        write_view.set(40u64);
        let write_test_value: u64 = write_view.get();
        assert_eq!(write_test_value, 40);
    }

    {
        let bytes_test = [1u8, 2u8, 3u8, 4u8];
        let test_bytes_from = &Reg::from(bytes_test.as_slice());
        let mut test_data_bytes = [0u8; 4];
        test_bytes_from.get_bytes(test_data_bytes.as_mut_slice());
        assert_eq!(bytes_test, test_data_bytes);

        let new_bytes_test = [100u8, 101u8, 102u8, 103u8];
        test_bytes_from.set(new_bytes_test.as_slice());
        test_bytes_from.get_bytes(test_data_bytes.as_mut_slice());
        assert_eq!(new_bytes_test, test_data_bytes);
    }
}

#[test]
fn channel_api() {
    let mut channel_store = ChannelStore::default();
    let mut channel_builder = ChannelBuilder::new(0usize);

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

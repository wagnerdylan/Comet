use comet::channel::{
    reg::{Reg, RegGetter, RegMutView, RegReadView, RegSetter},
    store::{ChannelStore, RegViewProducer},
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

#[test]
fn channel_api() {
    let mut channel_store: ChannelStore<10> = ChannelStore::default();
    let test_one_owner_tok =
        channel_store.register_write_channel("test1.test.channel", Reg::from(42u32));
    let _test_two_owner_tok =
        channel_store.register_write_channel("test2.test.channel", Reg::from(9000.0f64));
    let test_one_read_tok = channel_store.register_read_channel("test1.test.channel");

    {
        let mut_reg = channel_store.grab(&test_one_owner_tok);
        let mut_reg_val: u32 = mut_reg.get();
        assert_eq!(mut_reg_val, 42u32);

        mut_reg.set(10u32);
        let new_mut_reg_val: u32 = mut_reg.get();
        assert_eq!(new_mut_reg_val, 10u32);
    }
    {
        let read_reg = channel_store.grab(&test_one_read_tok);
        let reg_val: u32 = read_reg.get();
        assert_eq!(reg_val, 10u32);
    }
}

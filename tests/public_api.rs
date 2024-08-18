use comet::channel::reg::{Reg, RegGetter, RegMutView, RegReadView, RegSetter};

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

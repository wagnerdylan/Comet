use comet::channel::reg::{Reg, RegGetter, RegSetter};

extern crate comet;

#[test]
fn register_api() {
    let test_from: &Reg = &Reg::from(true);
    test_from.set(false);
    let test_value: bool = test_from.get();

    assert!(!test_value);
}

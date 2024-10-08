use core::{
    any::{self, TypeId},
    cell::RefCell,
    marker::PhantomData,
    mem,
};

use alloc::boxed::Box;
use downcast;
use dyn_clone::DynClone;

pub trait AnyClone: downcast::Any + DynClone {}
dyn_clone::clone_trait_object!(AnyClone);
downcast::downcast!(dyn AnyClone);

impl<T: Clone + downcast::Any> AnyClone for T {}

#[derive(Clone)]
pub struct Reg {
    reg_type: TypeId,
    data: RefCell<Box<dyn AnyClone>>,
}

impl Reg {
    pub fn new<T: 'static + AnyClone>(value: T) -> Self {
        Self {
            reg_type: TypeId::of::<T>(),
            data: RefCell::new(Box::new(value)),
        }
    }

    pub fn matches_type<T: 'static>(&self) -> Result<(), ()> {
        if TypeId::of::<T>() != self.reg_type {
            return Err(());
        }

        Ok(())
    }

    fn matches_type_panic<T: 'static>(&self) {
        if self.matches_type::<T>().is_err() {
            panic!(
                "Requested type of [{}] does not match register type.",
                any::type_name::<T>()
            )
        }
    }

    fn get<T: 'static + AnyClone + Clone>(&self) -> T {
        self.matches_type_panic::<T>();
        self.data.borrow().downcast_ref::<T>().unwrap().clone()
    }

    fn set<T: 'static>(&self, value: T) {
        self.matches_type_panic::<T>();
        let _ = mem::replace(self.data.borrow_mut().downcast_mut().unwrap(), value);
    }
}

pub struct RegReadView<'a, T: 'static + AnyClone + Clone> {
    reg: &'a Reg,
    phantom_marker: PhantomData<T>,
}

impl<'a, T: 'static + AnyClone + Clone> RegReadView<'a, T> {
    pub fn new(reg: &'a Reg) -> Self {
        Self {
            reg,
            phantom_marker: PhantomData,
        }
    }

    pub fn get(&self) -> T {
        self.reg.get()
    }
}

pub struct RegMutView<'a, T: 'static + AnyClone + Clone> {
    reg: &'a Reg,
    phantom_marker: PhantomData<T>,
}

impl<'a, T: 'static + AnyClone + Clone> RegMutView<'a, T> {
    pub fn new(reg: &'a Reg) -> Self {
        Self {
            reg,
            phantom_marker: PhantomData,
        }
    }

    pub fn get(&self) -> T {
        self.reg.get()
    }

    pub fn set(&self, value: T) {
        self.reg.set(value)
    }
}

#[cfg(test)]
mod unit_tests {
    use super::Reg;

    #[derive(Clone, PartialEq, Debug)]
    struct TestStruct(u8);

    #[test]
    fn test_reg_normal() {
        let reg = Reg::new(true);
        let mut get_reg: bool = reg.get();
        assert!(get_reg);
        reg.set(false);
        get_reg = reg.get();
        assert!(!get_reg);
    }

    #[test]
    fn test_reg_struct() {
        let reg = Reg::new(TestStruct(90));
        let mut get_reg: TestStruct = reg.get();
        assert_eq!(get_reg, TestStruct(90));
        reg.set(TestStruct(100));
        get_reg = reg.get();
        assert_eq!(get_reg, TestStruct(100));
    }

    #[test]
    #[should_panic(expected = "Requested type of [i32] does not match register type.")]
    fn test_set_reg_type_mismatch() {
        let reg = Reg::new(true);
        let get_reg: bool = reg.get();
        assert!(get_reg);
        reg.set(0);
    }

    #[test]
    #[should_panic(expected = "Requested type of [u8] does not match register type.")]
    fn test_get_reg_type_mismatch() {
        let reg = Reg::new(true);
        let _get_reg: u8 = reg.get();
    }
}

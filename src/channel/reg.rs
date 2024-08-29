use core::{
    any::{self, Any, TypeId},
    cell::RefCell,
    mem,
};

use alloc::boxed::Box;

pub struct Reg {
    reg_type: TypeId,
    data: RefCell<Box<dyn Any>>,
}

impl Reg {
    pub fn new<T: 'static>(value: T) -> Self {
        Self {
            reg_type: TypeId::of::<T>(),
            data: RefCell::new(Box::new(value)),
        }
    }

    fn get<T: 'static + Clone>(&self) -> T {
        if TypeId::of::<T>() != self.reg_type {
            panic!(
                "Requested type of [{}] does not match register type for get().",
                any::type_name::<T>()
            )
        }
        self.data.borrow().downcast_ref::<T>().unwrap().clone()
    }

    fn try_get<T: 'static + Clone>(&self) -> Result<T, ()> {
        if TypeId::of::<T>() != self.reg_type {
            return Err(());
        }
        Ok(self.get())
    }

    fn set<T: 'static>(&self, value: T) {
        if TypeId::of::<T>() != self.reg_type {
            panic!(
                "Requested type of [{}] does not match register type for set().",
                any::type_name::<T>()
            )
        }

        let _ = mem::replace(self.data.borrow_mut().downcast_mut().unwrap(), value);
    }

    fn try_set<T: 'static>(&self, value: T) -> Result<(), ()> {
        if TypeId::of::<T>() != self.reg_type {
            return Err(());
        }
        self.set(value);
        Ok(())
    }
}

pub struct RegReadView<'a> {
    reg: &'a Reg,
}

impl<'a> RegReadView<'a> {
    pub fn new(reg: &'a Reg) -> Self {
        Self { reg }
    }

    pub fn get<T: 'static + Clone>(&self) -> T {
        self.reg.get()
    }
}

pub struct RegMutView<'a> {
    reg: &'a Reg,
}

impl<'a> RegMutView<'a> {
    pub fn new(reg: &'a Reg) -> Self {
        Self { reg }
    }

    pub fn get<T: 'static + Clone>(&self) -> T {
        self.reg.get()
    }

    pub fn set<T: 'static>(&self, value: T) {
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
    #[should_panic(expected = "Requested type of [i32] does not match register type for set().")]
    fn test_set_reg_type_mismatch() {
        let reg = Reg::new(true);
        let get_reg: bool = reg.get();
        assert!(get_reg);
        reg.set(0);
    }

    #[test]
    #[should_panic(expected = "Requested type of [u8] does not match register type for get().")]
    fn test_get_reg_type_mismatch() {
        let reg = Reg::new(true);
        let _get_reg: u8 = reg.get();
    }

    #[test]
    fn test_try_get_set() {
        let reg = Reg::new(true);
        let get_reg: Result<bool, ()> = reg.try_get();
        assert!(get_reg.is_ok());
        assert!(get_reg.unwrap());

        let get_reg_u8: Result<u8, ()> = reg.try_get();
        assert!(get_reg_u8.is_err());

        assert!(reg.try_set(false).is_ok());
        assert!(reg.try_set(9u8).is_err());
    }
}

use core::cell::RefCell;

use alloc::vec::Vec;

#[derive(Debug, PartialEq)]
enum RegType {
    Bytes,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    F32,
    U64,
    I64,
    F64,
}

pub struct Reg {
    reg_type: RegType,
    data: RefCell<Vec<u8>>,
}

impl Reg {
    pub fn get_bytes(&self, out_slice: &mut [u8]) {
        assert_eq!(self.reg_type, RegType::Bytes);
        assert!(out_slice.len() == self.data.borrow().len());

        for (i, byte) in self.data.borrow().iter().enumerate() {
            out_slice[i] = *byte;
        }
    }

    pub fn try_get_bytes(&self, out_slice: &mut [u8]) -> Result<(), &'static str> {
        if self.reg_type != RegType::Bytes {
            return Err("Get type mismatch.");
        }
        self.get_bytes(out_slice);

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
}

pub struct RegMutView<'a> {
    reg: &'a Reg,
}

impl<'a> RegMutView<'a> {
    pub fn new(reg: &'a Reg) -> Self {
        Self { reg }
    }
}

pub trait RegSetter<T> {
    fn set(&self, value: T);

    fn try_set(&self, value: T) -> Result<(), &'static str>;
}

pub trait RegGetter<T> {
    fn get(&self) -> T;

    fn try_get(&self) -> Result<T, &'static str>;
}

macro_rules! reg_setter {
    ($prim_type:ty, $enum_type:expr) => {
        impl RegSetter<$prim_type> for Reg {
            fn set(&self, value: $prim_type) {
                assert_eq!(self.reg_type, $enum_type);
                self.data.borrow_mut().clear();
                self.data
                    .borrow_mut()
                    .extend_from_slice(&value.to_ne_bytes());
            }

            fn try_set(&self, value: $prim_type) -> Result<(), &'static str> {
                if self.reg_type != $enum_type {
                    return Err("Set type mismatch.");
                }
                self.set(value);

                Ok(())
            }
        }
    };
}

impl RegSetter<&[u8]> for Reg {
    fn set(&self, value: &[u8]) {
        assert_eq!(self.reg_type, RegType::Bytes);
        self.data.borrow_mut().clear();
        self.data.borrow_mut().extend_from_slice(value);
    }

    fn try_set(&self, value: &[u8]) -> Result<(), &'static str> {
        if self.reg_type != RegType::Bytes {
            return Err("Set type mismatch.");
        }
        self.set(value);

        Ok(())
    }
}

macro_rules! reg_from_type {
    ($prim_type:ty, $enum_type:expr) => {
        impl From<$prim_type> for Reg {
            fn from(value: $prim_type) -> Self {
                let data = value.to_ne_bytes();
                let mut data_vec = Vec::with_capacity(data.len());
                data_vec.extend_from_slice(&data);
                Reg {
                    reg_type: $enum_type,
                    data: RefCell::new(data_vec),
                }
            }
        }
    };
}

impl From<&[u8]> for Reg {
    fn from(value: &[u8]) -> Self {
        let mut data_vec = Vec::with_capacity(value.len());
        data_vec.extend_from_slice(value);
        Reg {
            reg_type: RegType::Bytes,
            data: RefCell::new(data_vec),
        }
    }
}

macro_rules! reg_getter {
    ($prim_type:ty, $enum_type:expr, $num_bytes:expr) => {
        impl RegGetter<$prim_type> for Reg {
            fn get(&self) -> $prim_type {
                assert_eq!(self.reg_type, $enum_type);
                let mut data_bytes: [u8; $num_bytes] = [0; $num_bytes];
                for (i, byte) in self.data.borrow().iter().enumerate() {
                    data_bytes[i] = *byte;
                }
                <$prim_type>::from_ne_bytes(data_bytes)
            }

            fn try_get(&self) -> Result<$prim_type, &'static str> {
                if self.reg_type != $enum_type {
                    return Err("Get type mismatch.");
                }
                Ok(self.get())
            }
        }
    };
}

macro_rules! reg_view_getter {
    ($prim_type:ty, $view_ident:ident) => {
        impl RegGetter<$prim_type> for $view_ident<'_> {
            fn get(&self) -> $prim_type {
                self.reg.get()
            }

            fn try_get(&self) -> Result<$prim_type, &'static str> {
                self.reg.try_get()
            }
        }
    };
}

macro_rules! reg_view_setter {
    ($prim_type:ty, $view_ident:ident) => {
        impl RegSetter<$prim_type> for $view_ident<'_> {
            fn set(&self, val: $prim_type) {
                self.reg.set(val);
            }

            fn try_set(&self, val: $prim_type) -> Result<(), &'static str> {
                self.reg.try_set(val)
            }
        }
    };
}

reg_setter!(u8, RegType::U8);
reg_from_type!(u8, RegType::U8);
reg_getter!(u8, RegType::U8, 1);
reg_view_getter!(u8, RegReadView);
reg_view_getter!(u8, RegMutView);
reg_view_setter!(u8, RegMutView);

reg_setter!(i8, RegType::I8);
reg_from_type!(i8, RegType::I8);
reg_getter!(i8, RegType::I8, 1);
reg_view_getter!(i8, RegReadView);
reg_view_getter!(i8, RegMutView);
reg_view_setter!(i8, RegMutView);

reg_setter!(u16, RegType::U16);
reg_from_type!(u16, RegType::U16);
reg_getter!(u16, RegType::U16, 2);
reg_view_getter!(u16, RegReadView);
reg_view_getter!(u16, RegMutView);
reg_view_setter!(u16, RegMutView);

reg_setter!(i16, RegType::I16);
reg_from_type!(i16, RegType::I16);
reg_getter!(i16, RegType::I16, 2);
reg_view_getter!(i16, RegReadView);
reg_view_getter!(i16, RegMutView);
reg_view_setter!(i16, RegMutView);

reg_setter!(u32, RegType::U32);
reg_from_type!(u32, RegType::U32);
reg_getter!(u32, RegType::U32, 4);
reg_view_getter!(u32, RegReadView);
reg_view_getter!(u32, RegMutView);
reg_view_setter!(u32, RegMutView);

reg_setter!(i32, RegType::I32);
reg_from_type!(i32, RegType::I32);
reg_getter!(i32, RegType::I32, 4);
reg_view_getter!(i32, RegReadView);
reg_view_getter!(i32, RegMutView);
reg_view_setter!(i32, RegMutView);

reg_setter!(f32, RegType::F32);
reg_from_type!(f32, RegType::F32);
reg_getter!(f32, RegType::F32, 4);
reg_view_getter!(f32, RegReadView);
reg_view_getter!(f32, RegMutView);
reg_view_setter!(f32, RegMutView);

reg_setter!(u64, RegType::U64);
reg_from_type!(u64, RegType::U64);
reg_getter!(u64, RegType::U64, 8);
reg_view_getter!(u64, RegReadView);
reg_view_getter!(u64, RegMutView);
reg_view_setter!(u64, RegMutView);

reg_setter!(i64, RegType::I64);
reg_from_type!(i64, RegType::I64);
reg_getter!(i64, RegType::I64, 8);
reg_view_getter!(i64, RegReadView);
reg_view_getter!(i64, RegMutView);
reg_view_setter!(i64, RegMutView);

reg_setter!(f64, RegType::F64);
reg_from_type!(f64, RegType::F64);
reg_getter!(f64, RegType::F64, 8);
reg_view_getter!(f64, RegReadView);
reg_view_getter!(f64, RegMutView);
reg_view_setter!(f64, RegMutView);

#[cfg(test)]
mod unit_tests {
    use crate::channel::reg::{RegGetter, RegMutView, RegSetter};

    use super::{Reg, RegReadView};

    #[test]
    #[should_panic(expected = "assertion `left == right` failed\n  left: U32\n right: U64")]
    fn test_reg_setter() {
        let reg = Reg::from(42u32);
        reg.set(32u32);
        let reg_val: u32 = reg.get();
        assert_eq!(reg_val, 32u32);

        let reg_panic = Reg::from(0u32);
        // This line is expected to panic due to internal type conversion.
        reg_panic.set(32u64);
    }

    #[test]
    fn test_reg_from_type() {
        let reg = Reg::from(16u8);
        let reg_val: u8 = reg.get();
        assert_eq!(reg_val, 16u8);
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed\n  left: U8\n right: U16")]
    fn test_reg_getter() {
        let reg = Reg::from(16u8);
        // This line is expected to panic due to internal type conversion.
        let _reg_val: u16 = reg.get();
    }

    #[test]
    fn test_reg_read_view() {
        let reg = Reg::from(16u8);
        let read_view = RegReadView::new(&reg);
        let reg_value: u8 = read_view.get();

        assert_eq!(reg_value, 16);
    }

    #[test]
    fn test_reg_mut_view() {
        let reg = Reg::from(16u8);
        let read_view = RegMutView::new(&reg);
        let reg_value: u8 = read_view.get();

        assert_eq!(reg_value, 16);

        read_view.set(8u8);
        let new_reg_value: u8 = read_view.get();
        assert_eq!(new_reg_value, 8);
    }

    #[test]
    #[should_panic(expected = "Get type mismatch.")]
    fn test_try_reg_get() {
        let reg = Reg::from(8u8);
        let _valid_val: u8 = reg.try_get().unwrap();

        let _invalid_val: i64 = reg.try_get().unwrap();
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: \"Set type mismatch.\""
    )]
    fn test_try_reg_set() {
        let reg = Reg::from(42.0f32);
        reg.try_set(50.0f32).unwrap();

        let reg_val: f32 = reg.get();
        assert_eq!(reg_val, 50.0f32);

        reg.try_set(4u8).unwrap();
    }

    #[test]
    fn test_get_bytes() {
        let bytes_test = [1u8, 2, 3, 4, 5];
        let reg = Reg::from(bytes_test.as_slice());
        let mut test_data_bytes = [0u8; 5];

        reg.get_bytes(&mut test_data_bytes);
        assert_eq!(bytes_test, test_data_bytes);

        test_data_bytes = [0u8; 5];
        reg.try_get_bytes(&mut test_data_bytes).unwrap();
    }
}

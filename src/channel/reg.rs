use core::cell::RefCell;

union RegData {
    bool: bool,
    u8: u8,
    i8: i8,
    u16: u16,
    i16: i16,
    u32: u32,
    i32: i32,
    f32: f32,
    u64: u64,
    i64: i64,
    f64: f64,
}

#[derive(Debug, PartialEq)]
enum RegType {
    BOOL,
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
    data: RefCell<RegData>,
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
    fn set(&self, val: T);
}

pub trait RegGetter<T> {
    fn get(&self) -> T;
}

macro_rules! reg_setter {
    ($prim_type:ty, $prim_id:ident, $enum_type:expr) => {
        impl RegSetter<$prim_type> for Reg {
            fn set(&self, val: $prim_type) {
                assert_eq!(self.reg_type, $enum_type);
                self.data.replace(RegData { $prim_id: val });
            }
        }
    };
}

macro_rules! reg_from_type {
    ($prim_type:ty, $prim_id:ident, $enum_type:expr) => {
        impl From<$prim_type> for Reg {
            fn from(value: $prim_type) -> Self {
                Reg {
                    reg_type: $enum_type,
                    data: RefCell::new(RegData { $prim_id: value }),
                }
            }
        }
    };
}

macro_rules! reg_getter {
    ($prim_type:ty, $prim_id:ident, $enum_type:expr) => {
        impl RegGetter<$prim_type> for Reg {
            fn get(&self) -> $prim_type {
                assert_eq!(self.reg_type, $enum_type);
                unsafe { self.data.borrow().$prim_id }
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
        }
    };
}

macro_rules! reg_view_setter {
    ($prim_type:ty, $view_ident:ident) => {
        impl RegSetter<$prim_type> for $view_ident<'_> {
            fn set(&self, val: $prim_type) {
                self.reg.set(val);
            }
        }
    };
}

reg_setter!(bool, bool, RegType::BOOL);
reg_from_type!(bool, bool, RegType::BOOL);
reg_getter!(bool, bool, RegType::BOOL);
reg_view_getter!(bool, RegReadView);
reg_view_getter!(bool, RegMutView);
reg_view_setter!(bool, RegMutView);

reg_setter!(u8, u8, RegType::U8);
reg_from_type!(u8, u8, RegType::U8);
reg_getter!(u8, u8, RegType::U8);
reg_view_getter!(u8, RegReadView);
reg_view_getter!(u8, RegMutView);
reg_view_setter!(u8, RegMutView);

reg_setter!(i8, i8, RegType::I8);
reg_from_type!(i8, i8, RegType::I8);
reg_getter!(i8, i8, RegType::I8);
reg_view_getter!(i8, RegReadView);
reg_view_getter!(i8, RegMutView);
reg_view_setter!(i8, RegMutView);

reg_setter!(u16, u16, RegType::U16);
reg_from_type!(u16, u16, RegType::U16);
reg_getter!(u16, u16, RegType::U16);
reg_view_getter!(u16, RegReadView);
reg_view_getter!(u16, RegMutView);
reg_view_setter!(u16, RegMutView);

reg_setter!(i16, i16, RegType::I16);
reg_from_type!(i16, i16, RegType::I16);
reg_getter!(i16, i16, RegType::I16);
reg_view_getter!(i16, RegReadView);
reg_view_getter!(i16, RegMutView);
reg_view_setter!(i16, RegMutView);

reg_setter!(u32, u32, RegType::U32);
reg_from_type!(u32, u32, RegType::U32);
reg_getter!(u32, u32, RegType::U32);
reg_view_getter!(u32, RegReadView);
reg_view_getter!(u32, RegMutView);
reg_view_setter!(u32, RegMutView);

reg_setter!(i32, i32, RegType::I32);
reg_from_type!(i32, i32, RegType::I32);
reg_getter!(i32, i32, RegType::I32);
reg_view_getter!(i32, RegReadView);
reg_view_getter!(i32, RegMutView);
reg_view_setter!(i32, RegMutView);

reg_setter!(f32, f32, RegType::F32);
reg_from_type!(f32, f32, RegType::F32);
reg_getter!(f32, f32, RegType::F32);
reg_view_getter!(f32, RegReadView);
reg_view_getter!(f32, RegMutView);
reg_view_setter!(f32, RegMutView);

reg_setter!(u64, u64, RegType::U64);
reg_from_type!(u64, u64, RegType::U64);
reg_getter!(u64, u64, RegType::U64);
reg_view_getter!(u64, RegReadView);
reg_view_getter!(u64, RegMutView);
reg_view_setter!(u64, RegMutView);

reg_setter!(i64, i64, RegType::I64);
reg_from_type!(i64, i64, RegType::I64);
reg_getter!(i64, i64, RegType::I64);
reg_view_getter!(i64, RegReadView);
reg_view_getter!(i64, RegMutView);
reg_view_setter!(i64, RegMutView);

reg_setter!(f64, f64, RegType::F64);
reg_from_type!(f64, f64, RegType::F64);
reg_getter!(f64, f64, RegType::F64);
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
}

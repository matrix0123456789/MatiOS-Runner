use crate::uuid::Uuid;

pub struct TypedValue {
    pub value_type: usize,
    pub value: usize,
}
impl TypedValue {
    pub fn null(value: bool) -> TypedValue {
        TypedValue {
            value_type: 0,
            value: 0,
        }
    }
    pub fn bool(value: bool) -> TypedValue {
        TypedValue {
            value_type: 1,
            value: value as usize,
        }
    }
    pub fn u8(value: u8) -> TypedValue {
        TypedValue {
            value_type: 2,
            value: value as usize,
        }
    }
    pub fn i8(value: i8) -> TypedValue {
        TypedValue {
            value_type: 3,
            value: value as usize,
        }
    }

    pub fn u16(value: u16) -> TypedValue {
        TypedValue {
            value_type: 4,
            value: value as usize,
        }
    }
    pub fn i16(value: i16) -> TypedValue {
        TypedValue {
            value_type: 5,
            value: value as usize,
        }
    }

    pub fn u32(value: u32) -> TypedValue {
        TypedValue {
            value_type: 6,
            value: value as usize,
        }
    }
    pub fn i32(value: i32) -> TypedValue {
        TypedValue {
            value_type: 7,
            value: value as usize,
        }
    }
    pub fn u64(value: u64) -> TypedValue {
        TypedValue {
            value_type: 8,
            value: value as usize,
        }
    }
    pub fn i64(value: i64) -> TypedValue {
        TypedValue {
            value_type: 9,
            value: value as usize,
        }
    }
    pub fn uuid(value: Uuid) -> TypedValue {
        TypedValue {
            value_type: 10,
            value: unsafe{Box::into_raw(Box::from(value)) as *const u8 as usize},
        }
    }
    pub fn string(value: String) -> TypedValue {
        TypedValue {
            value_type: 11,
            value: unsafe{Box::into_raw(Box::from(value)) as *const String as usize},
        }
    }
}

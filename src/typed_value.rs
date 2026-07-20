use crate::uuid::Uuid;
use std::alloc::alloc;
use std::collections::HashMap;
#[repr(C)]
pub struct TypedValue {
    pub value_type: usize,
    pub value: usize,
}
#[repr(C)]
pub struct KeyedTypedValue {
    pub key: String,
    pub value_type: usize,
    pub value: usize,
}
impl TypedValue {
    pub const fn null() -> TypedValue {
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
            value: unsafe { Box::into_raw(Box::from(value)) as *const u8 as usize },
        }
    }
    pub fn string(value: String) -> TypedValue {
        TypedValue {
            value_type: 11,
            value: unsafe { Box::into_raw(Box::from(value)) as *const String as usize },
        }
    }
    pub fn vector(values: &[TypedValue]) -> TypedValue {
        unsafe {
            let buff = alloc(
                core::alloc::Layout::from_size_align(
                    values.len() * core::mem::size_of::<TypedValue>()
                        + core::mem::size_of::<usize>(),
                    core::mem::align_of::<TypedValue>(),
                )
                .unwrap(),
            );
            (buff as *mut usize).write(values.len());
            let valuesPtr = (buff as *mut usize).offset(1) as *mut TypedValue;
            for i in 0..values.len() {
                valuesPtr
                    .offset(i as isize)
                    .write_volatile(values[i].clone())
            }
            TypedValue {
                value_type: 12,
                value: buff as usize,
            }
        }
    }
    pub fn get_as_u64(&self) -> u64 {
        if (self.value_type != 8) {
            panic!("Invalid value type");
        }
        return self.value as u64;
    }
    pub fn structure(values: &[KeyedTypedValue]) -> TypedValue {
        unsafe {
            let buff = alloc(
                core::alloc::Layout::from_size_align(
                    values.len() * core::mem::size_of::<KeyedTypedValue>()
                        + core::mem::size_of::<usize>(),
                    core::mem::align_of::<KeyedTypedValue>(),
                )
                .unwrap(),
            );
            (buff as *mut usize).write(values.len());
            let valuesPtr = (buff as *mut usize).offset(1) as *mut KeyedTypedValue;
            for i in 0..values.len() {
                valuesPtr
                    .offset(i as isize)
                    .write_volatile(values[i].clone())
            }
            TypedValue {
                value_type: 13,
                value: buff as usize,
            }
        }
    }
    pub fn get_as_structure(&self) -> HashMap<String, TypedValue> {
        if (self.value_type != 13) {
            panic!("Invalid value type");
        }
        let buff = self.value as *const u8;
        let len = unsafe { (buff as *const usize).read() };
        let valuesPtr = unsafe { ((buff as *const usize).offset(1)) as *const KeyedTypedValue };
        let mut ret = HashMap::new();
        for i in 0..len {
            let value = unsafe { valuesPtr.offset(i as isize).read() };
            ret.insert(
                value.key,
                TypedValue {
                    value_type: value.value_type,
                    value: value.value,
                },
            );
        }
        return ret;
    }
}
impl Clone for TypedValue {
    fn clone(&self) -> Self {
        return Self {
            value_type: self.value_type,
            value: self.value,
        };
    }
}
impl Clone for KeyedTypedValue {
    fn clone(&self) -> KeyedTypedValue {
        return KeyedTypedValue {
            value_type: self.value_type,
            value: self.value,
            key: String::from(self.key.as_str()),
        };
    }
}
impl KeyedTypedValue {
    pub fn from(key: String, value: TypedValue) -> KeyedTypedValue {
        return KeyedTypedValue {
            value_type: value.value_type,
            value: value.value,
            key: key,
        };
    }
}

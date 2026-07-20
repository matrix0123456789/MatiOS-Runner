use crate::syscalls::SyscallRequest;
use crate::uuid::Uuid;
use std::boxed::Box;
use std::string::String;
use std::vec;
use std::vec::Vec;
#[repr(C)]
pub struct GetResourceInfoV1Request {
    pub uuid: Uuid,
}
#[repr(C)]
pub struct GetResourceInfoV1Response {
    pub uuid: Uuid,
    pub methods:vec::Vec<String>,
    pub name: String,
    pub resource_type: Uuid,
    pub tags: vec::Vec<Uuid>,
    pub connected_resources: vec::Vec<Uuid>,
}
impl GetResourceInfoV1Request {
    pub fn create(uuid: Uuid) -> Box<SyscallRequest<Self>> {
        Box::new(SyscallRequest {
            size: size_of::<Self>(),
            uuid: crate::uuid::Uuid::from_u128(0xf60347f7_c312_48c2_9db4_0b5efb60db08),
            payload: Self { uuid },
        })
    }
}

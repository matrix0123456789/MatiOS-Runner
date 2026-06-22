use crate::syscalls::SyscallRequest;
use crate::uuid::Uuid;
use std::boxed::Box;
use std::string::String;
use std::vec::Vec;
pub struct GetResourceInfoV1Request {
    pub uuid: Uuid,
}

pub struct GetResourceInfoV1Response {
    pub uuid: Uuid,
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

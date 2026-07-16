use crate::syscalls::SyscallRequest;
use crate::uuid::Uuid;
use std::boxed::Box;
use std::string::String;
use std::vec::Vec;
#[repr(C)]
pub struct RequestResourceV1 {
    pub resource_type: Uuid,
    pub tags:Vec<Uuid>,

}
#[repr(C)]
pub struct RequestResourceV1Response {
    pub uuid: Uuid

}
impl RequestResourceV1 {
    pub fn create(resource_type: Uuid, owner: Uuid,tags:Vec<Uuid>) -> Box<SyscallRequest<Self>> {
        Box::new(SyscallRequest {
            size: size_of::<Self>(),
            uuid: crate::uuid::Uuid::from_u128(0x621e666e_fedb_42b2_8817_3df5780d9d0b)                ,
            payload: Self {
                resource_type,
                tags,
            },
        })
    }
}

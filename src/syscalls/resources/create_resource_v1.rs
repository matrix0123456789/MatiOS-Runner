use crate::syscalls::SyscallRequest;
use crate::uuid::Uuid;
use std::boxed::Box;
use std::string::String;
use std::vec::Vec;
#[repr(C)]
pub struct CreateResourceV1 {
    pub resource_type: Uuid,
    pub owner: Uuid,
    pub tags:Vec<Uuid>,

}
#[repr(C)]
pub struct CreateResourceV1Response {
    pub uuid: Uuid

}
impl CreateResourceV1 {
    pub fn create(resource_type: Uuid, owner: Uuid,tags:Vec<Uuid>) -> Box<SyscallRequest<Self>> {
        Box::new(SyscallRequest {
            size: size_of::<Self>(),
            uuid: crate::uuid::Uuid::from_u128(0xb2828475_e770_4bdc_86e0_695695d6bab0)                ,
            payload: Self {
                resource_type,
                owner,
                tags,
            },
        })
    }
}

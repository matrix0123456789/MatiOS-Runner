use crate::syscalls::SyscallRequest;
use crate::uuid::Uuid;
use std::boxed::Box;
use std::string::String;
use std::vec;
use std::vec::Vec;
#[repr(C)]
pub struct GetResourceByPathV1Request {
    pub path: String,
}
#[repr(C)]
pub struct GetResourceByPathV1Response {
    pub uuid: Uuid
}

use crate::syscalls::SyscallRequest;
use crate::uuid::Uuid;
use std::string::String;

pub struct PrintV1 {

    pub text:String
}
impl PrintV1 {
    pub fn create(text: String) -> SyscallRequest<Self> {
        SyscallRequest {
            size: size_of:: <Self>(),
            uuid: crate::uuid::Uuid::parse_str("7b16bee9-d0b8-4bd5-86d7-8225840ce006").unwrap(),
            payload: Self{text}
        }
    }
}
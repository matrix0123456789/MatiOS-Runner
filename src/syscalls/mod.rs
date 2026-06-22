use crate::uuid::Uuid;

#[repr(C)]
pub struct SyscallRequest<T> {
    pub size: usize,
    pub uuid: Uuid,
    pub payload: T,
}
#[repr(C)]
pub struct SyscallResponse<T> {
    pub size: usize,
    pub request_uuid: Uuid,
    pub payload: T,
}
pub mod debug;
pub mod process;

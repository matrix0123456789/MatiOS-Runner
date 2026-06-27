use crate::uuid::Uuid;
#[repr(C)]
pub struct CurrentProcessInfoV1Request {}

#[repr(C)]
pub struct CurrentProcessInfoV1Response {
    pub uuid: Uuid,
    pub name: String,
}


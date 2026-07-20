
use std::ops::Deref;
use crate::resource_local_registry::{Resource, RESOURCE_LOCAL_REGISTRY};
use crate::uuid::Uuid;
use std::sync::Arc;

pub mod desktop;
pub mod files;

pub const RESOURCE_BYTE_STREAM_ID: crate::uuid::Uuid =
    crate::uuid::Uuid::from_u128(0x964cb3b0_a12c_4a0b_ba85_c10cbdd2d416);
pub const RESOURCE_BYTE_STREAM_TAG_STDIN: crate::uuid::Uuid =
    crate::uuid::Uuid::from_u128(0xe312761d_5b12_4da3_a338_282374d2d642);
pub const RESOURCE_BYTE_STREAM_TAG_STDOUT: crate::uuid::Uuid =
    crate::uuid::Uuid::from_u128(0x6c582e01_48c5_45ce_a12d_f6bb16faf3c0);
pub const RESOURCE_BYTE_STREAM_TAG_STDERR: crate::uuid::Uuid =
    crate::uuid::Uuid::from_u128(0xafc00e60_6611_45a2_836e_a6d142517dfc);

pub const RESOURCE_WINDOW_ID: crate::uuid::Uuid =
    crate::uuid::Uuid::from_u128(0xf15e18c_bcbc_48da_95ed_f41c093bc849);
pub const RESOURCE_DESKTOP_ID: crate::uuid::Uuid =
    crate::uuid::Uuid::from_u128(0xe2e95ac2_c619_4767_8194_90e817fb3646);
pub const RESOURCE_FILESYSTEM_ITEM_ID: crate::uuid::Uuid =
    crate::uuid::Uuid::from_u128(0x3e1f8c5a_9b6d_4c2e_8f7a_7567eb785687);
pub const RESOURCE_FILESYSTEM_ITEM_ROOT_TAG: crate::uuid::Uuid =
    crate::uuid::Uuid::from_u128(0x3e1f8c5a_9b6d_4c2e_8f7a_1bff44d518e8);
pub const RESOURCE_FILESYSTEM_ITEM_CURRENT_DIR_TAG: crate::uuid::Uuid =
    crate::uuid::Uuid::from_u128(0x03071a33_9b6d_4c2e_8f7a_aea9b057a344);

pub fn get_resource_by_path(path: String) -> Option<Arc<Resource>> {
    if (path.contains(':')) {
        let protocol = path.split(':').next().unwrap();
        if (protocol == "matios") {
            let remaining = &path[protocol.len() + 1..];
            if (remaining == "currentProcess") {
                let processId = std::process::id();
                let process = unsafe { windows_sys::Win32::System::Threading::GetCurrentProcess() };
                let sytheticProcessId = 0x30312746_893c_4654_a9b5_000000000000;

                let registry = RESOURCE_LOCAL_REGISTRY.lock().unwrap();
                let resource =
                    registry.get(&Uuid::from_u128((sytheticProcessId + processId as u128)));
                return resource.cloned();
            } else {
                return None;
            }
        } else {
            return None;
        }
    } else {
        let uuid = Uuid::parse_str(path.as_str());
        if (uuid.is_ok()) {

            let registry = RESOURCE_LOCAL_REGISTRY.lock().unwrap();
            let resource = registry.get(&uuid.unwrap());
            if (resource.is_some()) {
                return resource.cloned();
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
}

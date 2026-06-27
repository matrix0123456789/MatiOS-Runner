use crate::resource_local_registry::{Resource, RESOURCE_LOCAL_REGISTRY};
use crate::uuid::Uuid;
use crate::{resource_local_registry, syscalls};
use core::panic::PanicInfo;
use std::alloc::Layout;
use std::collections::HashMap;
use std::rc::Rc;
use crate::typed_value::TypedValue;

pub struct ProcessStartInfo {
    processId: Uuid,
    debugPrint: extern "win64" fn(&str),
    debugPrintInt: extern "win64" fn(u64),
    debugPrintUuid: extern "win64" fn(Uuid),
    debugPanicRust: extern "win64" fn(&PanicInfo),
    allocate: extern "win64" fn(size: u64, align: u64) -> u64,
    syscallSync: extern "win64" fn(usize) -> usize,
}
impl ProcessStartInfo {
    pub fn getVerboseInfo() -> Self {
        extern "win64" fn debugPrint(x: &str) {
            println!("{}", x);
        }
        extern "win64" fn debugPanicRust(x: &PanicInfo) {
            vec![].push(x);
            println!("Panic inside");
        }
        extern "win64" fn debugPrintInt(x: u64) {
            println!("0x{:X} - {}", x, x);
        }
        extern "win64" fn debugPrintUuid(x: Uuid) {
            println!("{:X}", x.as_u128());
        }
        extern "win64" fn allocate(size: u64, align: u64) -> u64 {
            let all = unsafe {
                std::alloc::alloc(Layout::from_size_align_unchecked(
                    size as usize,
                    align as usize,
                )) as u64
            };
            return all;
        }
        Self::prepare_host_machine_resource();
        extern "win64" fn syscall_sync(req: usize) -> usize {
            return syscalls::syscall_sync(req);
        }
        return ProcessStartInfo {
            processId: Self::prepare_process_resource(),
            debugPrint: debugPrint,
            debugPrintInt: debugPrintInt,
            debugPrintUuid: debugPrintUuid,
            debugPanicRust: debugPanicRust,
            allocate: allocate,
            syscallSync: syscall_sync,
        };
    }
    pub fn getInfo() -> Self {
        extern "win64" fn debugPrint(x: &str) {}
        extern "win64" fn debugPanicRust(x: &PanicInfo) {
            panic!("Panic");
        }
        extern "win64" fn debugPrintInt(x: u64) {}
        extern "win64" fn debugPrintUuid(x: Uuid) {}
        extern "win64" fn allocate(size: u64, align: u64) -> u64 {
            let all = unsafe {
                std::alloc::alloc(Layout::from_size_align_unchecked(
                    size as usize,
                    align as usize,
                )) as u64
            };
            return all;
        }
        Self::prepare_host_machine_resource();
        extern "win64" fn syscall_sync(req: usize) -> usize {
            return syscalls::syscall_sync(req);
        }
        return ProcessStartInfo {
            processId: Self::prepare_process_resource(),
            debugPrint: debugPrint,
            debugPrintInt: debugPrintInt,
            debugPrintUuid: debugPrintUuid,
            debugPanicRust: debugPanicRust,
            allocate: allocate,
            syscallSync: syscall_sync,
        };
    }
    fn prepare_process_resource() -> Uuid {
        let processId = std::process::id() as u128;
        let sytheticProcessId = 0x30312746_893c_4654_a9b5_000000000000;
        let uuid = Uuid::from_u128(sytheticProcessId + processId);
        unsafe {
            RESOURCE_LOCAL_REGISTRY.lock().unwrap().insert(
                uuid,
                Resource {
                    uuid,
                    name: "Process".to_string(),
                    resource_type: Uuid::from_u128(0x541302e1_a401_40b8_8792_16a1fc4a54c5),
                    methods:HashMap::new()
                },
            );
        }
        return uuid;
    }
    fn prepare_host_machine_resource() -> Uuid {
        let uuid = Uuid::from_u128(0x5ee8f260_9a6e_4efc_b3be_eaef242c4cf0);
        unsafe {
            let mut methods:HashMap<String, fn()->TypedValue> = HashMap::new();
            methods.insert(String::from("getOsVersion"), ||TypedValue::string(String::from(os_version::detect().unwrap().to_string())));
            methods.insert(String::from("getCpuArchitecture"), ||TypedValue::string(String::from(std::env::consts::ARCH)));

            RESOURCE_LOCAL_REGISTRY.lock().unwrap().insert(
                uuid,
                Resource {
                    uuid,
                    name: "Host machine".to_string(),
                    resource_type: Uuid::from_u128(0x7fd422a9_36c6_45c9_b319_704d0c3d6001),
                    methods,
                },
            );
        }
        return uuid;
    }
}

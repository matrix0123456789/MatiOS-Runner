use crate::syscalls::debug::print_v1::PrintV1;
use crate::syscalls::process::current_process_info_v1::CurrentProcessInfoV1Response;
use crate::syscalls::resources::call_resource_method_v1::CallResourceMethodV1;
use crate::syscalls::resources::create_resource_v1::{CreateResourceV1, CreateResourceV1Response};
use crate::typedValue::TypedValue;
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
pub mod resources;
static mut StdOutResourceUuid: Option<Uuid> = None;

pub fn syscall_sync(req: usize) -> usize {
    let request = unsafe { &*(req as *const SyscallRequest<u8>) };
    unsafe {
        if ((*request).uuid == Uuid::from_u128(0x7b16bee9_d0b8_4bd5_86d7_8225840ce006)) {
            let requestTyped = unsafe { &*(req as *const SyscallRequest<PrintV1>) };
            print!("{}", requestTyped.payload.text);
        } else if ((*request).uuid == Uuid::from_u128(0xb2828475_e770_4bdc_86e0_695695d6bab0)) {
            let requestTyped = unsafe { &*(req as *const SyscallRequest<CreateResourceV1>) };
            if ((*requestTyped).payload.resource_type
                == Uuid::from_u128(0x964cb3b0_a12c_4a0b_ba85_c10cbdd2d416))
            {
                StdOutResourceUuid = Some(Uuid::from_u128(1)); //tmp, do random gen
            }
            let response = SyscallResponse {
                size: size_of::<CreateResourceV1Response>(),
                request_uuid: (*request).uuid.clone(),
                payload: CreateResourceV1Response {
                    uuid: StdOutResourceUuid.clone().unwrap(),
                },
            };
            return Box::into_raw(Box::from(response)) as usize;
        } else if ((*request).uuid == Uuid::from_u128(0xbce7baa2_c3e2_4f7f_9d42_42c94065f5f0)) {
            let requestTyped = unsafe { &*(req as *const SyscallRequest<CallResourceMethodV1>) };
            print!("{}", requestTyped.payload.method);
            if (requestTyped.payload.resource == StdOutResourceUuid.clone().unwrap()) {
                if (requestTyped.payload.method == "write") {
                    if(requestTyped.payload.args.value_type==11){
                        let string = unsafe { &*(requestTyped.payload.args.value as *const String) };
                        print!("{}", string);
                    }
                }
            }
        } else if ((*request).uuid == Uuid::from_u128(0x6ac0d646_72dc_4fe4_9fdc_f944f1a61491)) {
            let processId = std::process::id();
            let process = windows_sys::Win32::System::Threading::GetCurrentProcess();
            let sytheticProcessId = 0x30312746_893c_4654_a9b5_000000000000;
            let mut nameRaw = [0 as u16; 1024];
            let nameSize = Box::new(1024 as u32);
            let success = windows_sys::Win32::System::Threading::QueryFullProcessImageNameW(
                process,
                0,
                nameRaw.as_mut_ptr(),
                Box::into_raw(nameSize),
            );
            let name = String::from_utf16(nameRaw.as_slice()).unwrap();
            let response = SyscallResponse {
                size: size_of::<CurrentProcessInfoV1Response>(),
                request_uuid: (*request).uuid.clone(),
                payload: CurrentProcessInfoV1Response {
                    uuid: crate::uuid::Uuid::from_u128(sytheticProcessId + processId as u128),
                    name: name,
                },
            };
            return Box::into_raw(Box::from(response)) as usize;
        } else {
            println!("Unknow sycall, size: {}", (*request).size);
            println!(
                "uuid: {}",
                (*request)
                    .uuid
                    .as_bytes()
                    .iter()
                    .map(|x| format!("{:02X}", x))
                    .collect::<String>()
            );
        }
    }
    return 0;
}

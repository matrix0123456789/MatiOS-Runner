use std::collections::HashMap;
use crate::resource_local_registry::{Resource, RESOURCE_LOCAL_REGISTRY};
use crate::syscalls::debug::print_v1::PrintV1;
use crate::syscalls::process::current_process_info_v1::CurrentProcessInfoV1Response;
use crate::syscalls::resources::call_resource_method_v1::{
    CallResourceMethodV1, CallResourceMethodV1Response,
};
use crate::syscalls::resources::create_resource_v1::{CreateResourceV1, CreateResourceV1Response};
use crate::syscalls::resources::get_resource_info_v1::{
    GetResourceInfoV1Request, GetResourceInfoV1Response,
};
use crate::typed_value::TypedValue;
use crate::uuid::Uuid;
use std::ptr::null;
use windows_sys::core::{PCSTR, PCWSTR};
use windows_sys::{s, w};
use windows_sys::Win32::Foundation::{
    GetLastError, COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM,
};
use windows_sys::Win32::Graphics::Gdi::{CreateSolidBrush, UpdateWindow, ValidateRect, HBRUSH};
use windows_sys::Win32::System::Kernel::NULL64;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetModuleHandleW};
use windows_sys::Win32::UI::WindowsAndMessaging::{CreateWindowExA, CreateWindowExW, DefWindowProcA, DestroyWindow, DispatchMessageA, GetDesktopWindow, GetMessageA, LoadCursorW, PostQuitMessage, RegisterClassA, RegisterClassExA, RegisterClassExW, RegisterClassW, ShowWindow, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW, MINMAXINFO, SW_SHOW, WINDOW_EX_STYLE, WM_CLOSE, WM_GETMINMAXINFO, WNDCLASSA, WNDCLASSEXA, WNDCLASSEXW, WNDCLASSW, WS_BORDER, WS_EX_CLIENTEDGE, WS_OVERLAPPEDWINDOW, WS_VISIBLE};


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
mod host_machine;
pub mod process;
pub mod resources;
pub mod syscal_id;

static mut StdOutResourceUuid: Option<Uuid> = None;

pub fn syscall_sync(req: usize) -> usize {
    let request = unsafe { &*(req as *const SyscallRequest<u8>) };
    unsafe {
        if ((*request).uuid == syscal_id::PRINT_V1) {
            let requestTyped = unsafe { &*(req as *const SyscallRequest<PrintV1>) };
            print!("{}", requestTyped.payload.text);
        } 
        else if ((*request).uuid == syscal_id::CREATE_RESOURCE_V1) {
            let requestTyped = unsafe { &*(req as *const SyscallRequest<CreateResourceV1>) };
            if ((*requestTyped).payload.resource_type
                == Uuid::from_u128(0x964cb3b0_a12c_4a0b_ba85_c10cbdd2d416))
            {
                println!("Hello, world!");
                use windows_sys::{
                    Win32::Foundation::*, Win32::Graphics::Gdi::ValidateRect,
                    Win32::System::LibraryLoader::GetModuleHandleA, Win32::UI::WindowsAndMessaging::*, core::*,
                };

                extern "system" fn wndproc(
                    window: HWND,
                    message: u32,
                    wparam: WPARAM,
                    lparam: LPARAM,
                ) -> LRESULT {
                    unsafe {
                        match message {
                            WM_PAINT => {
                                println!("WM_PAINT");
                                ValidateRect(window, std::ptr::null());
                                0
                            }
                            WM_DESTROY => {
                                println!("WM_DESTROY");
                                PostQuitMessage(0);
                                0
                            }
                            _ => DefWindowProcA(window, message, wparam, lparam),
                        }
                    }
                }

                unsafe {
                    let instance = GetModuleHandleA(std::ptr::null());
                    debug_assert!(!instance.is_null());

                    let window_class = s!("window");

                    let wc = WNDCLASSA {
                        hCursor: LoadCursorW(core::ptr::null_mut(), IDC_ARROW),
                        hInstance: instance,
                        lpszClassName: window_class,
                        style: CS_HREDRAW | CS_VREDRAW,
                        lpfnWndProc: Some(wndproc),
                        cbClsExtra: 0,
                        cbWndExtra: 0,
                        hIcon: core::ptr::null_mut(),
                        hbrBackground: core::ptr::null_mut(),
                        lpszMenuName: std::ptr::null(),
                    };

                    let atom = RegisterClassA(&wc);
                    debug_assert!(atom != 0);

                    CreateWindowExA(
                        0,
                        window_class,
                        s!("This is a sample window"),
                        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        instance,
                        std::ptr::null(),
                    );

                    let mut message = std::mem::zeroed();

                    while GetMessageA(&mut message, core::ptr::null_mut(), 0, 0) != 0 {
                        DispatchMessageA(&message);
                    }
                }

                let windowId = Some(Uuid::from_u128(101)); //tmp, do random gen
                let response = SyscallResponse {
                    size: size_of::<CreateResourceV1Response>(),
                    request_uuid: (*request).uuid.clone(),
                    payload: CreateResourceV1Response {
                        uuid: windowId.clone().unwrap(),
                    },
                };
                return Box::into_raw(Box::from(response)) as usize;
            }
        } 
        else if ((*request).uuid == syscal_id::CALL_RESOURCE_METHOD_V1) {
            let requestTyped = unsafe { &*(req as *const SyscallRequest<CallResourceMethodV1>) };
            if (StdOutResourceUuid.is_some()
                && requestTyped.payload.resource == StdOutResourceUuid.clone().unwrap())
            {
                //todo move to resource method
                if (requestTyped.payload.method == "write") {
                    if (requestTyped.payload.args.value_type == 11) {
                        let string =
                            unsafe { &*(requestTyped.payload.args.value as *const String) };
                        print!("{}", string);
                    }
                }
            } else {
                let registry = RESOURCE_LOCAL_REGISTRY.lock().unwrap();
                let resource = registry.get(&requestTyped.payload.resource);
                if(resource.is_some()) {
                    if (resource.unwrap().methods.contains_key(&requestTyped.payload.method)) {
                        let method = resource.unwrap().methods.get(&requestTyped.payload.method).unwrap();
                        let result = method();
                        let response = SyscallResponse {
                            size: size_of::<CallResourceMethodV1Response>(),
                            request_uuid: (*request).uuid.clone(),
                            payload: CallResourceMethodV1Response { value: result },
                        };
                        return Box::into_raw(Box::from(response)) as usize;
                    }
                }
            }
        } 
        else if ((*request).uuid == syscal_id::CURRENT_PROCESS_INFO_V1) {
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
        } 
        else if ((*request).uuid == syscal_id::GET_RESOURCE_INFO_V1) {
            let requestTyped =
                unsafe { &*(req as *const SyscallRequest<GetResourceInfoV1Request>) };
            let uuid = requestTyped.payload.uuid;
            let registry = RESOURCE_LOCAL_REGISTRY.lock().unwrap();
            let resource = registry.get(&uuid).unwrap();

            let response = SyscallResponse {
                size: size_of::<GetResourceInfoV1Request>(),
                request_uuid: (*request).uuid.clone(),
                payload: GetResourceInfoV1Response {
                    uuid: (&resource).uuid,
                    methods: resource.methods.keys().map(|x| x.clone()).collect(),
                },
            };

            return Box::into_raw(Box::from(response)) as usize;
        } 
        else {
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
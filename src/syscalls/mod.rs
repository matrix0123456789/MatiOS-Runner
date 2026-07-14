use crate::resource_local_registry::{Resource, RESOURCE_LOCAL_REGISTRY};
use crate::resources::{RESOURCE_BYTE_STREAM_ID, RESOURCE_WINDOW_ID};
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
use once_cell::race::OnceBox;
use std::cell::{LazyCell, RefCell};
use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr::null;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use windows_sys::core::{PCSTR, PCWSTR};
use windows_sys::Win32::Foundation::{
    GetLastError, COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM,
};
use windows_sys::Win32::Graphics::Gdi::{BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateDIBSection, CreateSolidBrush, DeleteDC, EndPaint, GetDC, GetDIBits, InvalidateRect, ReleaseDC, SelectObject, SetPixel, UpdateWindow, ValidateRect, BITMAPINFO, BITMAPINFOHEADER, BI_BITFIELDS, BI_RGB, DIBSECTION, DIB_RGB_COLORS, HBRUSH, PAINTSTRUCT, RGBQUAD, SRCCOPY};
use windows_sys::Win32::System::Kernel::NULL64;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetModuleHandleW};
use windows_sys::Win32::System::Threading::Sleep;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExA, CreateWindowExW, DefWindowProcA, DestroyWindow, DispatchMessageA,
    GetDesktopWindow, GetMessageA, LoadCursorW, PostQuitMessage, RegisterClassA, RegisterClassExA,
    RegisterClassExW, RegisterClassW, ShowWindow, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW,
    MINMAXINFO, SW_SHOW, WINDOW_EX_STYLE, WM_CLOSE, WM_GETMINMAXINFO, WNDCLASSA, WNDCLASSEXA,
    WNDCLASSEXW, WNDCLASSW, WS_BORDER, WS_EX_CLIENTEDGE, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};
use windows_sys::{s, w};
use crate::bitmap::Color;

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
pub mod syscall_id;

static mut StdOutResourceUuid: Option<Uuid> = None;

static mut WindowHandle: HWND = 0 as HWND;
static mut WindowThreadPtr: usize = 0;
static mut channel: LazyCell<(
    Sender<Box<dyn FnOnce() + Send>>,
    Receiver<Box<dyn FnOnce() + Send>>,
)> = LazyCell::new(|| mpsc::channel::<Box<dyn FnOnce() + Send>>());

static mut lastWindowContent: LazyCell<HashMap<String, TypedValue>> =
    LazyCell::new(|| HashMap::new());

pub fn syscall_sync(req: usize) -> usize {
    let request = unsafe { &*(req as *const SyscallRequest<u8>) };
    unsafe {
        if ((*request).uuid == syscall_id::PRINT_V1) {
            let requestTyped = unsafe { &*(req as *const SyscallRequest<PrintV1>) };
            print!("{}", requestTyped.payload.text);
        } else if ((*request).uuid == syscall_id::CREATE_RESOURCE_V1) {
            let requestTyped = unsafe { &*(req as *const SyscallRequest<CreateResourceV1>) };
            if ((*requestTyped).payload.resource_type == RESOURCE_BYTE_STREAM_ID) {
                StdOutResourceUuid = Some(Uuid::from_u128(1)); //tmp, do random gen
                let mut methods: HashMap<String, fn(TypedValue) -> TypedValue> = HashMap::new();
                methods.insert("write".to_string(), |text| {
                    println!("{}", *(text.value as *const String));
                    TypedValue::null()
                });
                let mut registry = RESOURCE_LOCAL_REGISTRY.lock().unwrap();
                registry.insert(
                    StdOutResourceUuid.clone().unwrap(),
                    Resource {
                        uuid: StdOutResourceUuid.clone().unwrap(),
                        resource_type: RESOURCE_BYTE_STREAM_ID,
                        name: "StdOut".to_string(),
                        methods,
                    },
                );

                let response = SyscallResponse {
                    size: size_of::<CreateResourceV1Response>(),
                    request_uuid: (*request).uuid.clone(),
                    payload: CreateResourceV1Response {
                        uuid: StdOutResourceUuid.clone().unwrap(),
                    },
                };
                return Box::into_raw(Box::from(response)) as usize;
            } else if ((*requestTyped).payload.resource_type == RESOURCE_WINDOW_ID) {
                use windows_sys::{
                    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::ValidateRect,
                    Win32::System::LibraryLoader::GetModuleHandleA,
                    Win32::UI::WindowsAndMessaging::*,
                };
                let window_thread = thread::spawn(|| {
                    extern "system" fn wndproc(
                        window: HWND,
                        message: u32,
                        wparam: WPARAM,
                        lparam: LPARAM,
                    ) -> LRESULT {
                        unsafe {
                            match message {
                                WM_PAINT  => {
                                    // println!("WM_PAINT");
                                    if (lastWindowContent.contains_key("pixels")) {
                                        let mut pixels =
                                            lastWindowContent.get("pixels").unwrap().get_as_u64();
                                        let width =
                                            lastWindowContent.get("width").unwrap().get_as_u64();
                                        let height =
                                            lastWindowContent.get("height").unwrap().get_as_u64();

                                        let dc = GetDC(WindowHandle);

                                        let mut pixels2 = 0 as *mut _;
                                        let mut buf = 0 as *mut c_void;
                                        let mut bitmapinfo = BITMAPINFO {
                                            bmiHeader: BITMAPINFOHEADER {
                                                biSize: std::mem::size_of::<BITMAPINFOHEADER>()
                                                    as u32,
                                                biWidth: width as i32,
                                                biHeight: -(height as i32),
                                                biPlanes: 1,
                                                biBitCount: 32,
                                                biCompression: BI_RGB,
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        };
                                        let dibSection = CreateDIBSection(
                                            dc,
                                            &mut bitmapinfo,
                                            DIB_RGB_COLORS,
                                            //(pixels_ptr) as usize as *mut *mut _,
                                            &mut pixels2,
                                            0 as HANDLE,
                                            0,
                                        );
                                        (pixels2 as *mut u32)
                                            .copy_from(pixels as *mut u32, (width * height) as usize);
                                        for i in 0..(width * height) {
                                            (pixels2 as *mut u32).offset(i as isize).write_volatile(
                                                (pixels as *mut Color).offset(i as isize).read_volatile().to32bitInt()
                                            );
                                        }

                                        let hdcMemory = CreateCompatibleDC(dc);

                                        let mut ps: PAINTSTRUCT = Default::default();
                                        let hdc = BeginPaint(WindowHandle, &mut ps);
                                        let memdc = CreateCompatibleDC(hdc);
                                        SelectObject(memdc, dibSection);

                                        BitBlt(
                                            hdc, 0, 0, width as i32, height as i32, memdc, 0, 0,
                                            SRCCOPY,
                                        );
                                        DeleteDC(memdc);
                                        EndPaint(WindowHandle, &ps);
                                        ReleaseDC(WindowHandle, dc);
                                    }
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

                    let instance = GetModuleHandleA(std::ptr::null());
                    debug_assert!(!instance.is_null());

                    let window_class = s!("window");

                    let wc = WNDCLASSA {
                        hCursor: LoadCursorW(core::ptr::null_mut(), IDC_ARROW),
                        hInstance: instance,
                        lpszClassName: window_class,
                        style: CS_VREDRAW|CS_HREDRAW|CS_CLASSDC,
                        lpfnWndProc: Some(wndproc),
                        cbClsExtra: 0,
                        cbWndExtra: 0,
                        hIcon: core::ptr::null_mut(),
                        hbrBackground: core::ptr::null_mut(),
                        lpszMenuName: std::ptr::null(),
                    };

                    let atom = RegisterClassA(&wc);
                    debug_assert!(atom != 0);

                    let window_handle = CreateWindowExA(
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
                    WindowHandle = window_handle;

                    // Sleep(1000000);
                    let mut message = std::mem::zeroed();

                    while GetMessageA(&mut message, core::ptr::null_mut(), 0, 0) != 0 {
                        DispatchMessageA(&message);
                    }
                    // while let Ok(job) = channel.1.recv() {
                    //     job(); // wykonaj wstrzyknięty kod
                    // }
                });
                WindowThreadPtr = Box::into_raw(Box::from(window_thread)) as usize;
                let windowId = Some(Uuid::from_u128(101)); //tmp, do random gen

                let mut methods: HashMap<String, fn(TypedValue) -> TypedValue> = HashMap::new();
                methods.insert("writeBitmapBuffer".to_string(), |x| {
                    let structure = x.get_as_structure();
                    unsafe {
                        lastWindowContent = LazyCell::from(structure);
                    }

                    SetWindowPos(
                            WindowHandle,
                            core::ptr::null_mut(),
                            0,
                            0,
                            lastWindowContent.get("width").unwrap().get_as_u64() as i32,
                            lastWindowContent.get("height").unwrap().get_as_u64() as i32,
                            SWP_NOMOVE,
                        );
                    InvalidateRect(WindowHandle, 0 as *mut RECT, 1);
                    //     let mut message = std::mem::zeroed();
                    //     GetMessageA(&mut message, core::ptr::null_mut(), 0, 0);
                    //     DispatchMessageA(&message);
                    // }));
                    return TypedValue::null();
                });

                let mut registry = RESOURCE_LOCAL_REGISTRY.lock().unwrap();
                registry.insert(
                    windowId.clone().unwrap(),
                    Resource {
                        uuid: windowId.clone().unwrap(),
                        resource_type: RESOURCE_WINDOW_ID,
                        name: "Window".to_string(),
                        methods,
                    },
                );
                let response = SyscallResponse {
                    size: size_of::<CreateResourceV1Response>(),
                    request_uuid: (*request).uuid.clone(),
                    payload: CreateResourceV1Response {
                        uuid: windowId.clone().unwrap(),
                    },
                };
                return Box::into_raw(Box::from(response)) as usize;
            }
        } else if ((*request).uuid == syscall_id::CALL_RESOURCE_METHOD_V1) {
            let requestTyped = unsafe { &*(req as *const SyscallRequest<CallResourceMethodV1>) };

            let registry = RESOURCE_LOCAL_REGISTRY.lock().unwrap();
            let resource = registry.get(&requestTyped.payload.resource);
            if (resource.is_some()) {
                if (resource
                    .unwrap()
                    .methods
                    .contains_key(&requestTyped.payload.method))
                {
                    let method = resource
                        .unwrap()
                        .methods
                        .get(&requestTyped.payload.method)
                        .unwrap();
                    let result = method(requestTyped.payload.args.clone());
                    let response = SyscallResponse {
                        size: size_of::<CallResourceMethodV1Response>(),
                        request_uuid: (*request).uuid.clone(),
                        payload: CallResourceMethodV1Response { value: result },
                    };
                    return Box::into_raw(Box::from(response)) as usize;
                }
            }
        } else if ((*request).uuid == syscall_id::CURRENT_PROCESS_INFO_V1) {
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
        } else if ((*request).uuid == syscall_id::GET_RESOURCE_INFO_V1) {
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

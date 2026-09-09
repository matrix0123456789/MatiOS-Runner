use crate::bitmap::Color;
use crate::resource_local_registry::{Resource, RESOURCE_LOCAL_REGISTRY};
use crate::resources::{
    get_resource_by_path, RESOURCE_BYTE_STREAM_ID, RESOURCE_BYTE_STREAM_TAG_STDIN,
    RESOURCE_BYTE_STREAM_TAG_STDOUT, RESOURCE_DESKTOP_ID, RESOURCE_WINDOW_ID,
};
use crate::syscalls::debug::print_v1::PrintV1;
use crate::syscalls::process::current_process_info_v1::CurrentProcessInfoV1Response;
use crate::syscalls::resources::call_resource_method_v1::{
    CallResourceMethodV1, CallResourceMethodV1Response,
};
use crate::syscalls::resources::create_resource_v1::{CreateResourceV1, CreateResourceV1Response};
use crate::syscalls::resources::get_resource_by_path::{
    GetResourceByPathV1Request, GetResourceByPathV1Response,
};
use crate::syscalls::resources::get_resource_info_v1::{
    GetResourceInfoV1Request, GetResourceInfoV1Response,
};
use crate::syscalls::resources::request_resource_v1::{
    RequestResourceV1, RequestResourceV1Response,
};
use crate::typed_value::{KeyedTypedValue, TypedValue};
use crate::uuid::Uuid;
use once_cell::race::OnceBox;
use std::cell::{LazyCell, RefCell};
use std::collections::HashMap;
use std::ffi::c_void;
use std::io::{Read, Write};
use std::ptr::null;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, thread};
use windows_sys::core::{BOOL, PCSTR, PCWSTR};
use windows_sys::Win32::Foundation::{
    GetLastError, COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM,
};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateDIBSection,
    CreateSolidBrush, DeleteDC, EndPaint, GetDC, GetDIBits, InvalidateRect, ReleaseDC,
    SelectObject, SetPixel, UpdateWindow, ValidateRect, BITMAPINFO, BITMAPINFOHEADER, BI_BITFIELDS,
    BI_RGB, DIBSECTION, DIB_RGB_COLORS, HBRUSH, PAINTSTRUCT, RGBQUAD, SRCCOPY,
};
use windows_sys::Win32::System::Kernel::NULL64;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetModuleHandleW};
use windows_sys::Win32::System::Threading::Sleep;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExA, CreateWindowExW, DefWindowProcA, DestroyWindow, DispatchMessageA, EnumWindows,
    GetClassNameW, GetDesktopWindow, GetMessageA, GetWindowRect, GetWindowTextLengthW,
    GetWindowTextW, LoadCursorW, PostQuitMessage, RegisterClassA, RegisterClassExA,
    RegisterClassExW, RegisterClassW, ShowWindow, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW,
    MINMAXINFO, SW_SHOW, WINDOW_EX_STYLE, WM_CLOSE, WM_GETMINMAXINFO, WNDCLASSA, WNDCLASSEXA,
    WNDCLASSEXW, WNDCLASSW, WS_BORDER, WS_EX_CLIENTEDGE, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};
use windows_sys::{s, w};

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
static mut StdInResourceUuid: Option<Uuid> = None;

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
                if ((*requestTyped)
                    .payload
                    .tags
                    .contains(&RESOURCE_BYTE_STREAM_TAG_STDOUT))
                {
                    StdOutResourceUuid = Some(Uuid::from_u128(1)); //tmp, do random gen
                    let mut methods: HashMap<String, fn(TypedValue) -> TypedValue> = HashMap::new();
                    methods.insert("write".to_string(), |text| {
                        let strBytes = (*(text.value as *const String)).as_bytes();
                        console::Term::stdout().write(strBytes);
                        console::Term::stdout().flush();
                        TypedValue::null()
                    });
                    RESOURCE_LOCAL_REGISTRY.lock().unwrap().insert(
                        StdOutResourceUuid.clone().unwrap(),
                        Arc::new(Resource {
                            uuid: StdOutResourceUuid.clone().unwrap(),
                            resource_type: RESOURCE_BYTE_STREAM_ID,
                            name: "StdOut".to_string(),
                            methods,
                            ..Default::default()
                        }),
                    );

                    let response = SyscallResponse {
                        size: size_of::<CreateResourceV1Response>(),
                        request_uuid: (*request).uuid.clone(),
                        payload: CreateResourceV1Response {
                            uuid: StdOutResourceUuid.clone().unwrap(),
                        },
                    };
                    return Box::into_raw(Box::from(response)) as usize;
                } else if ((*requestTyped)
                    .payload
                    .tags
                    .contains(&RESOURCE_BYTE_STREAM_TAG_STDIN))
                {
                    StdInResourceUuid = Some(Uuid::from_u128(2)); //tmp, do random gen
                    let mut methods: HashMap<String, fn(TypedValue) -> TypedValue> = HashMap::new();
                    methods.insert("read".to_string(), |_| {
                        let char = console::Term::stdout().read_char().unwrap();
                        let string = String::from(char);
                        return TypedValue::string(string.to_string());
                    });
                    RESOURCE_LOCAL_REGISTRY.lock().unwrap().insert(
                        StdInResourceUuid.clone().unwrap(),
                        Arc::new(Resource {
                            uuid: StdInResourceUuid.clone().unwrap(),
                            resource_type: RESOURCE_BYTE_STREAM_ID,
                            name: "StdIn".to_string(),
                            methods,
                            ..Default::default()
                        }),
                    );

                    let response = SyscallResponse {
                        size: size_of::<CreateResourceV1Response>(),
                        request_uuid: (*request).uuid.clone(),
                        payload: CreateResourceV1Response {
                            uuid: StdInResourceUuid.clone().unwrap(),
                        },
                    };
                    return Box::into_raw(Box::from(response)) as usize;
                } else {
                    panic!("Unknown");
                }
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
                                WM_PAINT => {
                                    println!("WM_PAINT");
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
                                        (pixels2 as *mut u32).copy_from(
                                            pixels as *mut u32,
                                            (width * height) as usize,
                                        );
                                        for i in 0..(width * height) {
                                            (pixels2 as *mut u32)
                                                .offset(i as isize)
                                                .write_volatile(
                                                    (pixels as *mut Color)
                                                        .offset(i as isize)
                                                        .read_volatile()
                                                        .to32bitInt(),
                                                );
                                        }

                                        let hdcMemory = CreateCompatibleDC(dc);

                                        let mut ps: PAINTSTRUCT = Default::default();
                                        let hdc = BeginPaint(WindowHandle, &mut ps);
                                        let memdc = CreateCompatibleDC(hdc);
                                        SelectObject(memdc, dibSection);

                                        BitBlt(
                                            hdc,
                                            0,
                                            0,
                                            width as i32,
                                            height as i32,
                                            memdc,
                                            0,
                                            0,
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
                        style: CS_VREDRAW | CS_HREDRAW | CS_CLASSDC,
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
                let windowId = Some(Uuid::from_u128(0x303)); //tmp, do random gen

                let mut methods: HashMap<String, fn(TypedValue) -> TypedValue> = HashMap::new();
                methods.insert("getSize".to_string(), |x| {
                    let mut rect = RECT::default();
                    GetWindowRect(WindowHandle, &mut rect);
                    return TypedValue::structure(&[
                        KeyedTypedValue::from(
                            "width".to_string(),
                             TypedValue::u64((rect.right - rect.left) as u64),
                            //TypedValue::u64(300),
                        ),
                        KeyedTypedValue::from(
                            "height".to_string(),
                            TypedValue::u64((rect.bottom - rect.top) as u64),
                           // TypedValue::u64(500),
                        ),
                        KeyedTypedValue::from("pixelRatio".to_string(), TypedValue::u64(1)),
                    ]);
                });
                methods.insert("writeBitmapBuffer".to_string(), |x| {
                    let structure = x.get_as_structure();
                    unsafe {
                        lastWindowContent = LazyCell::from(structure);
                    }

                    // SetWindowPos(
                    //     WindowHandle,
                    //     core::ptr::null_mut(),
                    //     0,
                    //     0,
                    //     lastWindowContent.get("width").unwrap().get_as_u64() as i32,
                    //     lastWindowContent.get("height").unwrap().get_as_u64() as i32,
                    //     SWP_NOMOVE,
                    // );
                    //
                    // let mut rect = RECT::default();
                    // GetWindowRect(WindowHandle, &mut rect);

                    InvalidateRect(WindowHandle, 0 as *mut RECT, 1);
                    //     let mut message = std::mem::zeroed();
                    //     GetMessageA(&mut message, core::ptr::null_mut(), 0, 0);
                    //     DispatchMessageA(&message);
                    // }));
                    return TypedValue::null();
                });

                RESOURCE_LOCAL_REGISTRY.lock().unwrap().insert(
                    windowId.clone().unwrap(),
                    Arc::new(Resource {
                        uuid: windowId.clone().unwrap(),
                        resource_type: RESOURCE_WINDOW_ID,
                        name: "Window".to_string(),
                        methods,
                        ..Default::default()
                    }),
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
        } else if ((*request).uuid == syscall_id::REQUEST_RESOURCE_V1) {
            let requestTyped = unsafe { &*(req as *const SyscallRequest<RequestResourceV1>) };
            if ((*requestTyped).payload.resource_type == RESOURCE_DESKTOP_ID) {
                let desktopId = Some(Uuid::from_u128(0x70000000007)); //tmp, do random gen

                let registry = RESOURCE_LOCAL_REGISTRY.lock().unwrap();
                let resource = registry.get(&desktopId.clone().unwrap()).unwrap();
                let response = SyscallResponse {
                    size: size_of::<RequestResourceV1Response>(),
                    request_uuid: (*request).uuid.clone(),
                    payload: RequestResourceV1Response {
                        uuid: resource.uuid,
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
            let resource_option = registry.get(&uuid);
            let response;
            if (resource_option.is_some()) {
                let resource = resource_option.unwrap();

                response = SyscallResponse {
                    size: size_of::<GetResourceInfoV1Response>(),
                    request_uuid: (*request).uuid.clone(),
                    payload: GetResourceInfoV1Response {
                        uuid: (&resource).uuid,
                        name: (&resource).name.clone(),
                        resource_type: (&resource).resource_type,
                        tags: (&resource).tags.clone(),
                        methods: resource.methods.keys().map(|x| x.clone()).collect(),
                        connected_resources: (&resource)
                            .connected_resources
                            .iter()
                            .map(|x| x.uuid)
                            .collect(),
                    },
                };
            } else {
                response = SyscallResponse {
                    size: size_of::<GetResourceInfoV1Response>(),
                    request_uuid: (*request).uuid.clone(),
                    payload: GetResourceInfoV1Response {
                        uuid: Uuid::from_u128(0),
                        name: String::new(),
                        resource_type: Uuid::from_u128(0),
                        tags: vec![],
                        methods: vec![],
                        connected_resources: vec![],
                    },
                };
            }

            return Box::into_raw(Box::from(response)) as usize;
        } else if ((*request).uuid == syscall_id::GET_RESOURCE_BY_PATH_V1) {
            let requestTyped =
                unsafe { &*(req as *const SyscallRequest<GetResourceByPathV1Request>) };

            let resource = get_resource_by_path(requestTyped.payload.path.clone());

            let response = SyscallResponse {
                size: size_of::<GetResourceByPathV1Response>(),
                request_uuid: (*request).uuid.clone(),
                payload: GetResourceByPathV1Response {
                    uuid: if (resource.is_some()) {
                        resource.unwrap().uuid
                    } else {
                        Uuid::from_u128(0)
                    },
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

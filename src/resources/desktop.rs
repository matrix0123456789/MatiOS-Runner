use std::collections::HashMap;
use std::sync::Arc;
use windows_sys::core::BOOL;
use windows_sys::Win32::Foundation::{HWND, LPARAM, RECT};
use windows_sys::Win32::UI::WindowsAndMessaging::{EnumWindows, GetClassNameW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW};
use crate::resource_local_registry::{Resource, RESOURCE_LOCAL_REGISTRY};
use crate::resources::RESOURCE_DESKTOP_ID;
use crate::syscalls::resources::request_resource_v1::RequestResourceV1Response;
use crate::syscalls::SyscallResponse;
use crate::typed_value::{KeyedTypedValue, TypedValue};
use crate::uuid::Uuid;

pub fn prepare_current_desktop() {

        let desktopId = Some(Uuid::from_u128(0x70000000007)); //tmp, do random gen
        let mut methods: HashMap<String, fn(TypedValue) -> TypedValue> = HashMap::new();
        methods.insert("getWindows".to_string(), |text| {
            println!("getWindows");
            let mut windowsList: Box<Vec<TypedValue>> = Box::from(Vec::new());
            unsafe extern "system" fn enum_windows_callback(
                hwnd: HWND,
                lparam: LPARAM,
            ) -> BOOL {
                let mut windowsList = Box::from_raw(lparam as *mut Vec<TypedValue>);
                let mut rect = RECT::default();
                let aa = GetWindowRect(hwnd, &mut rect);
                println!(
                    "GetWindowRect: {}, {}, {}, {}",
                    rect.top, rect.left, rect.bottom, rect.right
                );

                let mut buffer = vec![0u16; 256];
                let written = GetClassNameW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);

                let className =
                    String::from_utf16_lossy(&buffer[..written.max(0) as usize]);
                println!("className: {}", className);

                let len = GetWindowTextLengthW(hwnd);
                let mut buffer2 = vec![0u16; (len.max(0) as usize) + 1];
                let written =
                    GetWindowTextW(hwnd, buffer2.as_mut_ptr(), buffer2.len() as i32);

                let windowName =
                    String::from_utf16_lossy(&buffer2[..written.max(0) as usize]);
                println!("windowName: {}", windowName);

                windowsList.push(TypedValue::structure(&[
                    KeyedTypedValue::from(
                        "className".to_string(),
                        TypedValue::string(className),
                    ),
                    KeyedTypedValue::from(
                        "windowName".to_string(),
                        TypedValue::string(windowName),
                    ),
                ]));

                1
            }
            let windowsListRaw = Box::into_raw(windowsList);
            unsafe {
                let windowsListCopy = Box::from_raw(windowsListRaw);
                EnumWindows(Some(enum_windows_callback), windowsListRaw as LPARAM);
                TypedValue::vector(windowsListCopy.as_slice())
            }
        });
    RESOURCE_LOCAL_REGISTRY.lock().unwrap().insert(
            desktopId.clone().unwrap(),
            Arc::new(Resource {
                uuid: desktopId.clone().unwrap(),
                resource_type: RESOURCE_DESKTOP_ID,
                name: "Desktop".to_string(),
                methods,
                ..Default::default()
            }),
        );



}
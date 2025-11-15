use std::arch::asm;
use std::{env, mem};
use std::alloc::{GlobalAlloc, Layout};
use std::io::Read;
use std::ops::Deref;

pub struct ProcessStartInfo {
    dummy: u64,
    debugPrint: extern "win64" fn(&str),
    debugPrintInt: extern "win64" fn(u64),
}
fn main() {
    {
        println!("{}", mem::size_of::<usize>());
        let mut filePath = String::new();
        let args: Vec<String> = env::args().collect();
        if (args.len() > 1) {
            filePath = args[1].clone();
        } else if (env::var("EXE").is_ok()) {
            filePath = env::var("EXE").unwrap();
        } else {
            return;
        }
        let function_pointer = loadExecutable(filePath);
        extern "win64" fn debugPrint(x:&str){
            println!("{}",x);
        }
        extern "win64" fn debugPrintInt(x:u64){
            println!("{}",x);
        }
        let info = Box::new(ProcessStartInfo {
            dummy: 123,
            debugPrint: debugPrint,
            debugPrintInt: debugPrintInt
        });
        let infoPrtr = Box::into_raw(info);
        let ret = function_pointer(infoPrtr);

        println!("Execution ended with return value: {}", ret);
    }
}
fn loadExecutable(filePath: String) -> extern "win64" fn(*mut ProcessStartInfo) -> u64 {
    println!("Hello, world!");

    println!("filePath, {}!", filePath);
    //open file stream
    let mut file = std::fs::File::open(&filePath).unwrap();
    //read binary whole file
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    //print file contents

    use windows_sys::Win32::System::Memory::{
        VirtualAlloc, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
    };

    let size: usize = 1024 * 1024;
    let ptr = unsafe {
        VirtualAlloc(
            std::ptr::null_mut(),
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        )
    };
    assert!(!ptr.is_null());
    //copy buffer to ptr
    let buffer_executable = ptr as *mut u8;
    unsafe {
        std::ptr::copy_nonoverlapping(buffer.as_ptr(), buffer_executable, buffer.len());
    }

    let function_pointer: extern "win64" fn(*mut ProcessStartInfo) -> u64 = unsafe {
        std::mem::transmute::<*mut u8, extern "win64" fn(*mut ProcessStartInfo) -> u64>(buffer_executable)
    };
    return function_pointer;
}






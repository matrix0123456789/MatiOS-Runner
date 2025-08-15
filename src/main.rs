use std::arch::asm;
use std::env;
use std::io::Read;

fn main() {
    {
        println!("{}", std::usize::MAX);
        let args: Vec<String> = env::args().collect();
        println!("Hello, world!");
        let mut filePath = String::new();
        if (args.len() > 1) {
            filePath = args[1].clone();
        } else if (env::var("EXE").is_ok()) {
            filePath = env::var("EXE").unwrap();
        } else {
            return;
        }

        println!("filePath, {}!", filePath);
        //open file stream
        let mut file = std::fs::File::open(&filePath).unwrap();
        //read binary whole file
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        //print file contents

        use windows_sys::Win32::System::Memory::{
            VirtualAlloc, MEM_COMMIT, MEM_RESERVE,
            PAGE_EXECUTE_READWRITE
        };

        let size: usize = 4096;
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
        let buffer_executable=ptr as *mut u8;
        unsafe {
            std::ptr::copy_nonoverlapping(buffer.as_ptr(), buffer_executable, buffer.len());
        }

//cast buffer_executable to function pointer
        let function_pointer = unsafe {
            std::mem::transmute::<*mut u8, fn()>(buffer_executable)
        };


        function_pointer();
        // unsafe {
        //     asm! {
        //     "jmp rax",
        //     in("rax") buffer_executable,
        //     options(noreturn)
        //     };
        // }

        println!("Execution ended");
    }
}

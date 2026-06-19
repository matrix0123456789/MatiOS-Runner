extern crate core;

use crate::elf_parser::{Elf64Shdr, ElfParser};
use core::panic::PanicInfo;
use std::alloc::{GlobalAlloc, Layout};
use std::any::Any;
use std::arch::asm;
use std::cell::RefCell;
use std::hash::Hash;
use std::io::Read;
use std::ops::{Deref, DerefMut};
use std::ptr::read_volatile;
use std::rc::Rc;
use std::{env, mem};

mod elf_parser;

pub struct ProcessStartInfo {
    dummy: u64,
    debugPrint: extern "win64" fn(&str),
    debugPrintInt: extern "win64" fn(u64),
    debugPanicRust: extern "win64" fn(&PanicInfo),
    allocate: extern "win64" fn(size: u64, align: u64) -> u64,
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
        extern "win64" fn debugPrint(x: &str) {
            println!("Printing ptr: 0x{:X}", x.as_ptr() as usize);
            println!("{}", x);
        }
        extern "win64" fn debugPanicRust(x: &PanicInfo) {
            println!("Panic inside");
        }
        extern "win64" fn debugPrintInt(x: u64) {
            println!("0x{:X} - {}", x, x);
        }
        extern "win64" fn allocate(size: u64, align: u64) -> u64 {
            let all = unsafe {
                std::alloc::alloc(Layout::from_size_align_unchecked(
                    size as usize,
                    align as usize,
                )) as u64
            };
            println!("allocatign {} alligned {} to {}", size, align, all);
            return all;
        }
        let info = Box::new(ProcessStartInfo {
            dummy: 123,
            debugPrint: debugPrint,
            debugPrintInt: debugPrintInt,
            debugPanicRust: debugPanicRust,
            allocate: allocate,
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

    let sections = ElfParser::parse(buffer.as_slice());
    // let mut text_buffer = Vec::new();
    // let mut rela_dyn_buffer: Vec<u8> = Vec::new();
    // let mut text_header = Elf64Shdr {
    //     sh_name: 0,
    //     sh_type: 0,
    //     sh_flags: 0,
    //     sh_addr: 0,
    //     sh_offset: 0,
    //     sh_size: 0,
    //     sh_link: 0,
    //     sh_info: 0,
    //     sh_addralign: 0,
    //     sh_entsize: 0,
    // };
    let mut start = i64::MAX;
    let mut end: i64 = 0;

    for section in &sections {
        if ((section.header.sh_flags & 0x02) > 0) {
            if ((section.header.sh_addr as i64) < start) {
                start = section.header.sh_addr as i64;
            }
            if (section.header.sh_addr as i64 + section.header.sh_size as i64 > end) {
                end = section.header.sh_addr as i64 + section.header.sh_size as i64;
            }
        }
    }
    let mut text_offset = 0;
    let mut allocated_memory = vec![0 as u8; (end - start) as usize];
    for section in sections {
        // if (section.name == ".text") {
        //     text_buffer = section.data;
        //     text_header = section.header;
        // } else if (section.name == ".rela.dyn") {
        //     rela_dyn_buffer = section.data;
        // }

        if ((section.header.sh_flags & 0x02) > 0) {
            let offset = section.header.sh_addr as usize - start as usize;
            let size = section.header.sh_size as usize;
            allocated_memory[offset..offset + size].copy_from_slice(&section.data);
            if (section.name == ".text") {
                text_offset = offset;
            }
        }
    }
    // let rela_dyn = rela_dyn_buffer.as_ptr() as *const u64;
    // for i in 0..(rela_dyn_buffer.len() / 24) as isize {
    //     unsafe {
    //         let offset = read_volatile(rela_dyn.byte_offset(i * 24 + 0));
    //         let info = read_volatile(rela_dyn.byte_offset(i * 24 + 8));
    //         let addent = read_volatile(rela_dyn.byte_offset(i * 24 + 16));
    //         if (offset >= (text_header).sh_addr
    //             && offset < (text_header).sh_addr + text_buffer.len() as u64)
    //         {
    //             println!("{} -> {}", offset, addent);
    //         }
    //         if (addent >= (text_header).sh_addr
    //             && addent < (text_header).sh_addr + text_buffer.len() as u64)
    //         {
    //             println!("{} -> {}", offset, addent);
    //         }
    //     }
    // }

    //print file contents

    use windows_sys::Win32::System::Memory::{
        VirtualAlloc, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
    };

    let size: usize = allocated_memory.len();
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
        std::ptr::copy_nonoverlapping(
            allocated_memory.as_ptr(),
            buffer_executable,
            allocated_memory.len(),
        );
    }

    let function_pointer: extern "win64" fn(*mut ProcessStartInfo) -> u64 = unsafe {
        std::mem::transmute::<*mut u8, extern "win64" fn(*mut ProcessStartInfo) -> u64>(
            (buffer_executable.byte_offset(text_offset as isize)),
        )
    };
    println!("loaded to pointer: {:X}", function_pointer as u64);
    return function_pointer;
}

extern crate core;

use crate::elf_parser::{Elf64Shdr, ElfParser};
use crate::process_start_info::ProcessStartInfo;
use crate::syscalls::debug::print_v1::PrintV1;
use crate::syscalls::process::current_process_info_v1::CurrentProcessInfoV1Response;
use crate::syscalls::{SyscallRequest, SyscallResponse};
use core::panic::PanicInfo;
use std::alloc::{alloc, GlobalAlloc, Layout};
use std::any::Any;
use std::arch::asm;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Read;
use std::ops::{Deref, DerefMut};
use std::ptr::read_volatile;
use std::rc::Rc;
use std::{env, mem, process};
use crate::resources::desktop::prepare_current_desktop;
use crate::resources::files::prepare_filesystem;

pub mod bitmap;
mod elf_parser;
pub mod host_machine;
pub mod process_start_info;
pub mod resource_local_registry;
pub mod resources;
pub mod syscalls;
pub mod typed_value;
pub mod uuid;

fn main() -> Result<(), std::io::Error> {
    {
        let mut filePath = String::new();
        let mut args: Vec<String> = env::args().collect();
        let mut verbose = true; //tmp true, change to false
        let mut load_local_environment = false;
        for i in 0..args.len() {
            let arg = &args[i];
            if arg == "--verbose" || arg == "-V" {
                verbose = true;
                println!("Verbose mode enabled");
                args.remove(i);
                break;
            }
        }
        for i in 0..args.len() {
            let arg = &args[i];
            if arg == "--localEnvironment" || arg == "-L" {
                load_local_environment = true;
                args.remove(i);
                break;
            }
        }
        if (args.len() > 1) {
            filePath = args[1].clone();
        } else if (env::var("EXE").is_ok()) {
            filePath = env::var("EXE").unwrap();
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No executable specified",
            ));
        }
        let function_pointer = loadExecutable(filePath);
        let info: ProcessStartInfo;
        if verbose {
            info = ProcessStartInfo::getVerboseInfo();
        } else {
            info = ProcessStartInfo::getInfo();
        }

        if load_local_environment {
            prepare_current_desktop();
            prepare_filesystem();
        }
        let infoPrtr = Box::into_raw(Box::from(info));
        let ret = function_pointer(infoPrtr);

        if (verbose) {
            println!("Execution ended with return value: {}", ret);
        }
        if (ret == 0) {
            return Ok(());
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Execution ended with return value: {}", ret),
            ));
        }
    }
}
fn loadExecutable(filePath: String) -> extern "win64" fn(*mut ProcessStartInfo) -> u64 {
    let mut file = std::fs::File::open(&filePath).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let sections = ElfParser::parse(buffer.as_slice());
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
    for section in sections.iter() {
        if ((section.header.sh_flags & 0x02) > 0) {
            let offset = section.header.sh_addr as usize - start as usize;
            let size = section.header.sh_size as usize;
            allocated_memory[offset..offset + size].copy_from_slice(&section.data);
            if (section.name == ".text_start") {
                text_offset = offset;
            }
            println!(
                "Loaded section {} at offset {:x} with size {:x} and flags {:x} and type {:x}",
                section.name, offset, size, section.header.sh_flags, section.header.sh_type
            );
        }
    }

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
    let buffer_executable = ptr as *mut u8;
    unsafe {
        std::ptr::copy_nonoverlapping(
            allocated_memory.as_ptr(),
            buffer_executable,
            allocated_memory.len(),
        );
    }
    for section in sections.iter() {
        if (section.header.sh_type == 4) {
            println!("RELA section");
            let datau64: Vec<u64> = unsafe { std::mem::transmute(section.data.clone()) };
            for i in 0..(section.header.sh_size / 8 / 3) as usize {
                unsafe {
                    let r_offset = datau64[i * 3];
                    let r_info = datau64[i * 3 + 1];
                    let r_addend = datau64[i * 3 + 2];
                    (buffer_executable as *mut u64)
                        .byte_offset(r_offset as isize)
                        .write_volatile(buffer_executable as u64 + r_addend);
                }
            }
        }
        if (section.header.sh_type == 9) {
            println!("REL section")
        }
    }
    print!("buffer_executable: {:p}\n", buffer_executable);
    print!("text_offset: {:x}\n", text_offset);
    let function_pointer: extern "win64" fn(*mut ProcessStartInfo) -> u64 = unsafe {
        std::mem::transmute::<*mut u8, extern "win64" fn(*mut ProcessStartInfo) -> u64>(
            (buffer_executable.byte_offset(text_offset as isize)),
        )
    };
    return function_pointer;
}

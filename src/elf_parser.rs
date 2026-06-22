use std::io::Read;
use std::mem;

pub struct ElfParser {}

impl ElfParser {
    pub(crate) fn parse(buffer: &[u8]) -> Vec<ElfSection> {
        // ELF header
        let header_buffer = &buffer[0..mem::size_of::<Elf64Ehdr>()];
        let header: &Elf64Ehdr = unsafe { &*(header_buffer.as_ptr() as *const Elf64Ehdr) };

        if &header.e_ident[0..4] != b"\x7FELF" {
            panic!("Not an ELF file");
        }

        // Sekcja z nazwami
        let name_offset =
            header.e_shoff as usize + (header.e_shstrndx as usize * header.e_shentsize as usize);
        let name_buffer = &buffer[name_offset..name_offset + mem::size_of::<Elf64Shdr>()];
        let shstr: &Elf64Shdr = unsafe { &*(name_buffer.as_ptr() as *const Elf64Shdr) };

        // Wczytaj string table
        let mut strtab =
            &buffer[shstr.sh_offset as usize..shstr.sh_offset as usize + shstr.sh_size as usize];
        let mut sections = Vec::new();
        // Iteruj po sekcjach
        for i in 0..header.e_shnum {
            let offset = header.e_shoff as usize + (i as usize * header.e_shentsize as usize);
            let buf_section = &buffer[offset..offset + mem::size_of::<Elf64Shdr>()];
            let shdr: &Elf64Shdr = unsafe { &*(buf_section.as_ptr() as *const Elf64Shdr) };

            let name_offset = shdr.sh_name as usize;
            let name = strtab[name_offset..].split(|&c| c == 0).next().unwrap();
            let name_str = String::from_utf8_lossy(name);
            sections.push(ElfSection {
                name: name_str.to_string(),
                data: Vec::from(
                    &buffer
                        [shdr.sh_offset as usize..shdr.sh_offset as usize + shdr.sh_size as usize],
                ),
                header: shdr.clone(),
            });
        }
        return sections;
    }
}

impl ElfParser {}

#[repr(C)]
#[derive(Debug)]
struct Elf64Ehdr {
    e_ident: [u8; 16], // Magic number and other info
    e_type: u16,       // Object file type
    e_machine: u16,    // Architecture
    e_version: u32,    // Object file version
    e_entry: u64,      // Entry point virtual address
    e_phoff: u64,      // Program header table file offset
    e_shoff: u64,      // Section header table file offset
    e_flags: u32,      // Processor-specific flags
    e_ehsize: u16,     // ELF header size in bytes
    e_phentsize: u16,  // Program header table entry size
    e_phnum: u16,      // Program header table entry count
    e_shentsize: u16,  // Section header table entry size
    e_shnum: u16,      // Section header table entry count
    e_shstrndx: u16,   // Section header string table index
}

#[repr(C)]
#[derive(Debug)]
pub struct Elf64Shdr {
    pub sh_name: u32,
    pub sh_type: u32,
    pub sh_flags: u64,
    pub sh_addr: u64,
    pub sh_offset: u64,
    pub sh_size: u64,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: u64,
    pub sh_entsize: u64,
}

pub struct ElfSection {
    pub name: String,
    pub data: Vec<u8>,
    pub header: Elf64Shdr,
}
impl Clone for ElfSection {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            data: self.data.clone(),
            header: self.header.clone(),
        }
    }
}

impl Clone for Elf64Shdr {
    fn clone(&self) -> Self {
        Self {
            sh_name: self.sh_name,
            sh_type: self.sh_type,
            sh_flags: self.sh_flags,
            sh_addr: self.sh_addr,
            sh_offset: self.sh_offset,
            sh_size: self.sh_size,
            sh_link: self.sh_link,
            sh_info: self.sh_info,
            sh_addralign: self.sh_addralign,
            sh_entsize: self.sh_entsize,
        }
    }
}

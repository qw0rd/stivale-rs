//! This module contains the definitions for stivale2 boot protocol.
//!
//! This Documentation contains miminal information about the protocol and provides brief
//! documentation for helper functions and methods.
//!
//! For detailed documentation, Visit the official [docs](https://github.com/stivale/stivale/blob/master/STIVALE2.md)
//!
//!
//!If you want a full working example, check the osdev's Stivale bare bones example.
//!Everything will work exactly the same way.
//!
//!# Example
//!```
//! use stivale_rs::v2::{
//!    Stivale2Header, Stivale2HeaderTagFrameBuffer, Stivale2HeaderTagTerminal, Stivale2Struct,
//!    Stivale2StructTagTerminal, STIVALE2_HEADER_TAG_FRAMEBUFFER_ID, STIVALE2_HEADER_TAG_TERMINAL_ID,
//!    STIVALE2_STRUCT_TAG_TERMINAL_ID,
//!   };
//!
//! static STACK: [u8; 4096] = [0; 4096];
//! static _term: Stivale2HeaderTagTerminal = Stivale2HeaderTagTerminal {
//!    identifier: STIVALE2_HEADER_TAG_TERMINAL_ID,
//!    next: core::ptr::null(),
//!    flags: 0,
//!   };
//!
//! #[no_mangle]
//! extern "C" fn _start(info: *const Stivale2Struct) {
//!    let info = unsafe { &*info };
//!    let _t = info.get_tag(STIVALE2_STRUCT_TAG_TERMINAL_ID);
//!
//!    if _t.is_null() {
//!         unsafe {
//!             loop {
//!             asm!("hlt");
//!             }
//!         }
//!    }
//!    let term = _t as *const Stivale2StructTagTerminal;
//!    let term = unsafe { &*term };
//!
//!    let print = term.get_term_func();
//!
//!    let brand = core::str::from_utf8(&info.bootloader_brand).unwrap();
//!    let version = core::str::from_utf8(&info.bootloader_version).unwrap();
//!
//!    print(brand);
//!    print("\n");
//!    print(version);
//!
//!     unsafe {
//!        loop {
//!            asm!("mov rax, 0xAA");
//!            asm!("hlt");
//!        }
//!    }
//! }
//!
//!
//! ````

/// The kernel executable shall have a section .stivale2hdr which will contain the header that the
/// bootloader will parse. The following header should be initalized as static and should be linked
/// as section `.stivale2hdr`
#[repr(C, packed)]
pub struct Stivale2Header {
    pub entry_point: *const (),

    /// The stack pointer should be cast to *const ().
    /// # Example
    /// ```
    /// static STACK : [u8;4096] = [0;4096];
    /// let ptr = &STACK[4095] as *const u8 as *const ();
    /// ```
    pub stack: *const (),

    pub flags: u64,

    /// Pointer to start of a linked list.
    ///
    /// Should be cast into `*const ()`.
    pub tags: *const (),
}

impl Stivale2Header {
    /// Create a stivale2 header structure which should be linked to the ELF Section
    /// ".stivale2hdr".
    ///
    /// # Example
    ///
    /// ```
    ///static STACK : [u8;4096] = [0;4096];
    ///
    ///#[used]
    ///#[link_section = ".stivale2hdr"]
    ///static hdr : Stivale2Header = Stivale2Header::new(core::ptr::null(),&stack, 1 << 0 , core::mem::null());
    ///
    /// ```
    pub const fn new<const SIZE: usize>(
        entry_point: *const (),
        stack: &[u8; SIZE],
        flags: u64,
        tags: *const (),
    ) -> Self {
        Stivale2Header {
            entry_point,
            stack: &stack[SIZE - 1] as *const u8 as *const (),
            flags,
            tags,
        }
    }
}

unsafe impl Sync for Stivale2Header {}
unsafe impl Send for Stivale2Header {}

#[repr(C, packed)]
pub struct Stivale2Tag {
    pub identifier: u64,
    pub next: *const (),
}

unsafe impl Sync for Stivale2Tag {}
unsafe impl Send for Stivale2Tag {}

pub const STIVALE2_HEADER_TAG_FRAMEBUFFER_ID: u64 = 0x3ecc1bc43d0f7971;

#[repr(C, packed)]
pub struct Stivale2HeaderTagFrameBuffer {
    pub identifier: u64,
    pub next: *const (),
    pub framebuffer_width: u16,
    pub framebuffer_height: u16,
    pub framebuffer_bpp: u16,
}

unsafe impl Sync for Stivale2HeaderTagFrameBuffer {}
unsafe impl Send for Stivale2HeaderTagFrameBuffer {}

pub const STIVALE2_HEADER_TAG_TERMINAL_ID: u64 = 0xa85d499b1823be72;

#[repr(C, packed)]
pub struct Stivale2HeaderTagTerminal {
    pub identifier: u64,
    pub next: *const (),
    pub flags: u64,
}

unsafe impl Sync for Stivale2HeaderTagTerminal {}
unsafe impl Send for Stivale2HeaderTagTerminal {}

#[repr(C, packed)]
pub struct Stivale2Struct {
    pub bootloader_brand: [u8; 64],
    pub bootloader_version: [u8; 64],
    pub tags: u64,
}

impl Stivale2Struct {
    /// Get a tag from the info passed on by the bootloader.
    ///
    /// Returned pointer should be checked for validity and cast into desired structure if valid.
    ///
    /// # Examples
    ///
    /// ```
    ///let tag = _info.get_tag(0xc2b3f4c3233b0974);
    ///
    ///if tag.is_null() {
    ///    // handle case
    ///} else {
    ///    let term = tag as *const Stivale2StructTagTerminal;
    ///}
    /// ```
    pub fn get_tag(&self, id: u64) -> *const () {
        let mut current = self.tags as *const ();

        loop {
            // If the linked list is null or the id is not found in the list.
            if current.is_null() {
                return core::ptr::null();
            }

            let _c = current as *const Stivale2Tag;

            unsafe {
                if (*_c).identifier == id {
                    return current;
                }
                current = (*_c).next as *const ();
            }
        }
    }
}

pub const STIVALE2_STRUCT_TAG_TERMINAL_ID: u64 = 0xc2b3f4c3233b0974;

#[repr(C, packed)]
pub struct Stivale2StructTagTerminal {
    pub identifier: u64,
    pub next: u64,
    pub flags: u32,
    pub cols: u16,
    pub rows: u16,
    pub term_write: u64,
}

impl Stivale2StructTagTerminal {
    pub fn get_term_func(&self) -> impl Fn(&str) {
        let _t = self.term_write as *const ();

        let _term_func =
            unsafe { core::mem::transmute::<*const (), extern "C" fn(*const i8, u64)>(_t) };

        move |txt| {
            _term_func(txt.as_ptr() as *const i8, txt.len() as u64);
        }
    }
}

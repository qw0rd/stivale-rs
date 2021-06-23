//! This module contains the definitions for stivale2 boot protocol.
//!
//! This Documentation contains miminal information about the protocol and provides brief
//! documentation for helper functions and methods.
//!
//! For detailed documentation, Visit the official [docs](https://github.com/stivale/stivale/blob/master/STIVALE2.md)
//!

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
    /// ```no_run
    ///static STACK : [u8;4096] = [0;4096];
    ///
    ///#[used]
    ///#[link_section = ".stivale2hdr"]
    ///static hdr : stivale_rs::v2::Stivale2Header = stivale_rs::v2::Stivale2Header::new(core::ptr::null(),&STACK, 1 << 0 , core::ptr::null());
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
    /// Please take a look at other convenient functions provided like
    /// `Stivale2Struct::get_terminal` and use them over directly using `get_tag`.
    /// # Examples
    ///
    /// ```no_run
    ///
    ///fn entry(info: stivale_rs::v2::Stivale2Struct) {
    ///     let tag = info.get_tag(0xc2b3f4c3233b0974);
    ///     if let Some(tag) = info.get_tag(0xc2b3f4c3233b0974) {
    ///         // use tag and cast it to desired structure.
    ///         let term = tag as *const stivale_rs::v2::Stivale2StructTagTerminal;
    ///     } else {
    ///         // handle case when tag not given by bootloader.
    ///     }
    /// }
    /// ```
    pub fn get_tag(&self, id: u64) -> Option<*const ()> {
        let mut current = self.tags as *const ();

        loop {
            // If the linked list is null or the id is not found in the list.
            if current.is_null() {
                return None;
            }

            let _c = current as *const Stivale2Tag;

            unsafe {
                if (*_c).identifier == id {
                    return Some(current);
                }
                current = (*_c).next as *const ();
            }
        }
    }

    /// Get a immutable reference to terminal info passed on by bootloader.
    pub fn get_terminal<'a>(&self) -> Option<&'a Stivale2StructTagTerminal> {
        let term = match self.get_tag(STIVALE2_STRUCT_TAG_TERMINAL_ID) {
            Some(term) => term,
            None => {
                return None;
            }
        };

        let term = term as *const Stivale2StructTagTerminal;
        let term = unsafe { &*term };
        Some(term)
    }

    /// Get framebuffer info.
    pub fn get_framebuffer<'a>(&self) -> Option<&'a Stivale2StructTagFramebuffer> {
        let fb = match self.get_tag(STIVALE2_STRUCT_TAG_FRAMEBUFFER_ID) {
            Some(fb) => fb,
            None => return None,
        };

        let fb = fb as *const Stivale2StructTagFramebuffer;
        let fb = unsafe { &*fb };

        Some(fb)
    }

    /// Get a tag using id as type T.
    ///
    /// **Warning**: This will definitely result in a crash if passed the wrong type. Please make
    /// sure you use the real type that is attributed to the id.
    pub fn _get<'a, T>(&self, id: u64) -> Option<&'a T> {
        let tag = self.get_tag(id);

        match tag {
            Some(t) => {
                let tag_ref = t as *const T;
                Some(unsafe { &*tag_ref })
            }
            None => None,
        }
    }
}

pub const STIVALE2_STRUCT_TAG_FRAMEBUFFER_ID: u64 = 0x506461d2950408fa;

#[repr(C, packed)]
pub struct Stivale2StructTagFramebuffer {
    pub identifier: u64,
    pub next: u64,
    pub framebuffer_addr: u64,
    pub framebuffer_width: u16,
    pub framebuffer_height: u16,
    pub framebuffer_pitch: u16,
    pub framebuffer_bpp: u16,
    pub memory_model: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
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
    /// Get a `term` function to print to framebuffer.
    /// It is provided by stivale2 boot protocol.
    pub fn get_term_func(&self) -> impl Fn(&str) {
        let _t = self.term_write as *const ();

        let _term_func =
            unsafe { core::mem::transmute::<*const (), extern "C" fn(*const i8, u64)>(_t) };

        move |txt| {
            _term_func(txt.as_ptr() as *const i8, txt.len() as u64);
        }
    }
}

/// This tag reports to the kernel the command line string that was passed to it by the bootloader.
#[repr(C, packed)]
pub struct Stivale2StructTagCmdline {
    pub identifier: u64,
    pub next: u64,
    pub cmdline: u64,
}

#[repr(C, packed)]
pub struct Stivale2StructTagMemmap {
    pub identifier: u64,
    pub next: u64,
    pub entries: u64,
    pub memmap: *const Stivale2MMapEntry,
}

#[repr(C, packed)]
pub struct Stivale2MMapEntry {
    pub base: u64,
    pub length: u64,
    pub r#type: u32,
    pub unsed: u32,
}

pub enum Stivale2MMapType {
    Usable = 1,
    Reserved,
    ACPIReclaimable,
    ACPINvs,
    BadMemory,
    BootloaderReclaimable = 0x1000,
    KernelAndModules = 0x1001,
    Framebuffer = 0x1002,
}

/// This tag reports to the kernel the current UNIX epoch, as per RTC.
#[repr(C, packed)]
pub struct Stivale2StructTagEpoch {
    pub identifier: u64,
    pub next: u64,
    pub epoch: u64,
}

/// This tag reports to the kernel info about the firmware.
#[repr(C, packed)]
pub struct Stivale2StructTagFirmware {
    pub identifier: u64,
    pub next: u64,
    pub flags: u64,
}

/// This tag provides the kernel with a pointer to the EFI system table if available.
#[repr(C, packed)]
pub struct Stivale2StructTagEFISystemTable {
    pub identifier: u64,
    pub next: u64,
    pub system_table: u64,
}

/// This tag provides the kernel with a pointer to a copy the raw executable file of the kernel
/// that the bootloader loaded.
#[repr(C, packed)]
pub struct Stivale2StructTagKernelFile {
    pub identifier: u64,
    pub next: u64,
    pub kernel_file: u64,
}

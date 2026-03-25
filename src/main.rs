//! framebuffer tangram demo for rCore tutorial ch1
//!
#![no_std]
#![no_main]
#![cfg_attr(target_arch = "riscv64", deny(warnings, missing_docs))]
#![cfg_attr(not(target_arch = "riscv64"), allow(dead_code))]

use tg_sbi::{console_putchar, shutdown};

#[cfg(target_arch = "riscv64")]
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
unsafe extern "C" fn _start() -> ! {
    const STACK_SIZE: usize = 4096;
    #[unsafe(link_section = ".bss.uninit")]
    static mut STACK: [u8; STACK_SIZE] = [0u8; STACK_SIZE];
    core::arch::naked_asm!(
        "la sp, {stack} + {stack_size}",
        "j  {main}",
        stack_size = const STACK_SIZE,
        stack = sym STACK,
        main = sym rust_main,
    )
}

#[cfg(target_arch = "riscv64")]
const FDT_MAGIC: u32 = 0xD00D_FEED;
#[cfg(target_arch = "riscv64")]
const FDT_BEGIN_NODE: u32 = 0x1;
#[cfg(target_arch = "riscv64")]
const FDT_END_NODE: u32 = 0x2;
#[cfg(target_arch = "riscv64")]
const FDT_PROP: u32 = 0x3;
#[cfg(target_arch = "riscv64")]
const FDT_NOP: u32 = 0x4;
#[cfg(target_arch = "riscv64")]
const FDT_END: u32 = 0x9;
#[cfg(target_arch = "riscv64")]
const FW_CFG_BASE: usize = 0x1010_0000;
#[cfg(target_arch = "riscv64")]
const FW_CFG_SELECTOR: usize = FW_CFG_BASE + 0x08;
#[cfg(target_arch = "riscv64")]
const FW_CFG_DATA8: usize = FW_CFG_BASE;
#[cfg(target_arch = "riscv64")]
const FW_CFG_DMA: usize = FW_CFG_BASE + 0x10;
#[cfg(target_arch = "riscv64")]
const FW_CFG_FILE_DIR: u16 = 0x0019;
#[cfg(target_arch = "riscv64")]
const FW_CFG_DMA_CTL_ERROR: u32 = 0x01;
#[cfg(target_arch = "riscv64")]
const FW_CFG_DMA_CTL_SELECT: u32 = 0x08;
#[cfg(target_arch = "riscv64")]
const FW_CFG_DMA_CTL_WRITE: u32 = 0x10;
#[cfg(target_arch = "riscv64")]
const RAMFB_WIDTH: usize = 800;
#[cfg(target_arch = "riscv64")]
const RAMFB_HEIGHT: usize = 600;
#[cfg(target_arch = "riscv64")]
const RAMFB_STRIDE: usize = RAMFB_WIDTH * 4;
#[cfg(target_arch = "riscv64")]
const RAMFB_SIZE: usize = RAMFB_STRIDE * RAMFB_HEIGHT;
#[cfg(target_arch = "riscv64")]
const DRM_FORMAT_XRGB8888: u32 = 0x3432_5258;

#[cfg(target_arch = "riscv64")]
#[derive(Clone, Copy)]
enum PixelFormat {
    X8R8G8B8,
    A8R8G8B8,
}

#[cfg(target_arch = "riscv64")]
#[derive(Clone, Copy)]
struct Framebuffer {
    addr: usize,
    width: usize,
    height: usize,
    stride_px: usize,
    format: PixelFormat,
}

#[cfg(target_arch = "riscv64")]
#[repr(C, align(4096))]
struct RamfbBuffer([u8; RAMFB_SIZE]);

#[cfg(target_arch = "riscv64")]
#[unsafe(link_section = ".bss.uninit")]
static mut RAMFB_BUFFER: RamfbBuffer = RamfbBuffer([0; RAMFB_SIZE]);

#[cfg(target_arch = "riscv64")]
#[repr(C)]
struct FwCfgDmaAccess {
    control: u32,
    length: u32,
    address: u64,
}

#[cfg(target_arch = "riscv64")]
#[repr(C, packed)]
struct RamfbCfg {
    addr: u64,
    fourcc: u32,
    flags: u32,
    width: u32,
    height: u32,
    stride: u32,
}

#[cfg(target_arch = "riscv64")]
impl Framebuffer {
    fn pack(self, rgb: u32) -> u32 {
        match self.format {
            PixelFormat::X8R8G8B8 => rgb & 0x00ff_ffff,
            PixelFormat::A8R8G8B8 => 0xff00_0000 | (rgb & 0x00ff_ffff),
        }
    }

    fn put_pixel(self, x: usize, y: usize, rgb: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let offset = y
            .saturating_mul(self.stride_px)
            .saturating_add(x)
            .saturating_mul(core::mem::size_of::<u32>());
        let ptr = (self.addr.saturating_add(offset)) as *mut u32;
        unsafe { ptr.write_volatile(self.pack(rgb)) }
    }

    fn clear(self, rgb: u32) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.put_pixel(x, y, rgb);
            }
        }
    }
}

#[cfg(target_arch = "riscv64")]
#[derive(Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[cfg(target_arch = "riscv64")]
#[derive(Clone, Copy)]
struct Piece {
    color: u32,
    points: [Point; 4],
    vertices: usize,
}

#[cfg(target_arch = "riscv64")]
const COLORS: [u32; 7] = [
    0xF94144, 0xF3722C, 0xF9C74F, 0x90BE6D, 0x43AA8B, 0x577590, 0x9D4EDD,
];

#[cfg(target_arch = "riscv64")]
const O_PIECES: [Piece; 7] = [
    Piece {
        color: COLORS[0],
        points: [
            Point { x: 0, y: 35 },
            Point { x: 35, y: 0 },
            Point { x: 35, y: 35 },
            Point { x: 0, y: 0 },
        ],
        vertices: 3,
    },
    Piece {
        color: COLORS[1],
        points: [
            Point { x: 0, y: 65 },
            Point { x: 35, y: 65 },
            Point { x: 35, y: 100 },
            Point { x: 0, y: 0 },
        ],
        vertices: 3,
    },
    Piece {
        color: COLORS[2],
        points: [
            Point { x: 65, y: 0 },
            Point { x: 100, y: 35 },
            Point { x: 65, y: 35 },
            Point { x: 0, y: 0 },
        ],
        vertices: 3,
    },
    Piece {
        color: COLORS[3],
        points: [
            Point { x: 65, y: 65 },
            Point { x: 82, y: 48 },
            Point { x: 100, y: 65 },
            Point { x: 0, y: 0 },
        ],
        vertices: 3,
    },
    Piece {
        color: COLORS[4],
        points: [
            Point { x: 65, y: 65 },
            Point { x: 82, y: 82 },
            Point { x: 100, y: 65 },
            Point { x: 0, y: 0 },
        ],
        vertices: 3,
    },
    Piece {
        color: COLORS[5],
        points: [
            Point { x: 35, y: 0 },
            Point { x: 65, y: 0 },
            Point { x: 65, y: 30 },
            Point { x: 35, y: 30 },
        ],
        vertices: 4,
    },
    Piece {
        color: COLORS[6],
        points: [
            Point { x: 35, y: 70 },
            Point { x: 65, y: 70 },
            Point { x: 55, y: 100 },
            Point { x: 25, y: 100 },
        ],
        vertices: 4,
    },
];

#[cfg(target_arch = "riscv64")]
const S_PIECES: [Piece; 7] = [
    Piece {
        color: COLORS[0],
        points: [
            Point { x: 0, y: 0 },
            Point { x: 45, y: 0 },
            Point { x: 0, y: 45 },
            Point { x: 0, y: 0 },
        ],
        vertices: 3,
    },
    Piece {
        color: COLORS[1],
        points: [
            Point { x: 55, y: 55 },
            Point { x: 100, y: 55 },
            Point { x: 100, y: 100 },
            Point { x: 0, y: 0 },
        ],
        vertices: 3,
    },
    Piece {
        color: COLORS[2],
        points: [
            Point { x: 25, y: 50 },
            Point { x: 50, y: 25 },
            Point { x: 75, y: 50 },
            Point { x: 0, y: 0 },
        ],
        vertices: 3,
    },
    Piece {
        color: COLORS[3],
        points: [
            Point { x: 55, y: 0 },
            Point { x: 100, y: 0 },
            Point { x: 100, y: 25 },
            Point { x: 0, y: 0 },
        ],
        vertices: 3,
    },
    Piece {
        color: COLORS[4],
        points: [
            Point { x: 0, y: 75 },
            Point { x: 0, y: 100 },
            Point { x: 45, y: 100 },
            Point { x: 0, y: 0 },
        ],
        vertices: 3,
    },
    Piece {
        color: COLORS[5],
        points: [
            Point { x: 35, y: 20 },
            Point { x: 60, y: 20 },
            Point { x: 60, y: 45 },
            Point { x: 35, y: 45 },
        ],
        vertices: 4,
    },
    Piece {
        color: COLORS[6],
        points: [
            Point { x: 40, y: 55 },
            Point { x: 70, y: 55 },
            Point { x: 55, y: 80 },
            Point { x: 25, y: 80 },
        ],
        vertices: 4,
    },
];

#[cfg(target_arch = "riscv64")]
fn read_be_u32(addr: usize) -> u32 {
    let ptr = addr as *const u8;
    let b0 = unsafe { ptr.read_volatile() as u32 };
    let b1 = unsafe { ptr.add(1).read_volatile() as u32 };
    let b2 = unsafe { ptr.add(2).read_volatile() as u32 };
    let b3 = unsafe { ptr.add(3).read_volatile() as u32 };
    (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
}

#[cfg(target_arch = "riscv64")]
fn read_be_u64(addr: usize) -> u64 {
    ((read_be_u32(addr) as u64) << 32) | (read_be_u32(addr + 4) as u64)
}

#[cfg(target_arch = "riscv64")]
fn align4(v: usize) -> usize {
    (v + 3) & !3
}

#[cfg(target_arch = "riscv64")]
fn cstr_eq_at(strings_base: usize, off: usize, target: &[u8]) -> bool {
    let mut i = 0usize;
    loop {
        let b = unsafe { ((strings_base + off + i) as *const u8).read_volatile() };
        if i < target.len() {
            if b != target[i] {
                return false;
            }
        } else {
            return b == 0;
        }
        i += 1;
    }
}

#[cfg(target_arch = "riscv64")]
fn compatible_has_simple_framebuffer(data: usize, len: usize) -> bool {
    let target = b"simple-framebuffer";
    let mut start = 0usize;
    while start < len {
        let mut end = start;
        while end < len {
            let b = unsafe { ((data + end) as *const u8).read_volatile() };
            if b == 0 {
                break;
            }
            end += 1;
        }
        if end - start == target.len() {
            let mut ok = true;
            let mut i = 0usize;
            while i < target.len() {
                let b = unsafe { ((data + start + i) as *const u8).read_volatile() };
                if b != target[i] {
                    ok = false;
                    break;
                }
                i += 1;
            }
            if ok {
                return true;
            }
        }
        start = end + 1;
    }
    false
}

#[cfg(target_arch = "riscv64")]
fn parse_simple_framebuffer(dtb_pa: usize) -> Option<Framebuffer> {
    if read_be_u32(dtb_pa) != FDT_MAGIC {
        return None;
    }
    let totalsize = read_be_u32(dtb_pa + 4) as usize;
    let off_dt_struct = read_be_u32(dtb_pa + 8) as usize;
    let off_dt_strings = read_be_u32(dtb_pa + 12) as usize;
    let struct_base = dtb_pa + off_dt_struct;
    let strings_base = dtb_pa + off_dt_strings;
    let struct_end = dtb_pa + totalsize;

    let mut p = struct_base;
    let mut depth = 0usize;
    let mut is_fb = false;
    let mut addr = 0usize;
    let mut width = 0usize;
    let mut height = 0usize;
    let mut stride = 0usize;
    let mut format = PixelFormat::X8R8G8B8;

    while p + 4 <= struct_end {
        let token = read_be_u32(p);
        p += 4;
        match token {
            FDT_BEGIN_NODE => {
                while p < struct_end {
                    let b = unsafe { (p as *const u8).read_volatile() };
                    p += 1;
                    if b == 0 {
                        break;
                    }
                }
                p = align4(p);
                depth = depth.saturating_add(1);
                is_fb = false;
                addr = 0;
                width = 0;
                height = 0;
                stride = 0;
                format = PixelFormat::X8R8G8B8;
            }
            FDT_END_NODE => {
                if is_fb && addr != 0 && width != 0 && height != 0 && stride != 0 {
                    return Some(Framebuffer {
                        addr,
                        width,
                        height,
                        stride_px: stride / 4,
                        format,
                    });
                }
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    break;
                }
            }
            FDT_PROP => {
                let len = read_be_u32(p) as usize;
                let nameoff = read_be_u32(p + 4) as usize;
                p += 8;
                let data = p;
                p = align4(p + len);

                if cstr_eq_at(strings_base, nameoff, b"compatible")
                    && compatible_has_simple_framebuffer(data, len)
                {
                    is_fb = true;
                } else if cstr_eq_at(strings_base, nameoff, b"reg") {
                    if len >= 16 {
                        addr = read_be_u64(data) as usize;
                    } else if len >= 8 {
                        addr = read_be_u32(data) as usize;
                    }
                } else if cstr_eq_at(strings_base, nameoff, b"width") && len >= 4 {
                    width = read_be_u32(data) as usize;
                } else if cstr_eq_at(strings_base, nameoff, b"height") && len >= 4 {
                    height = read_be_u32(data) as usize;
                } else if cstr_eq_at(strings_base, nameoff, b"stride") && len >= 4 {
                    stride = read_be_u32(data) as usize;
                } else if cstr_eq_at(strings_base, nameoff, b"format") && len >= 8 {
                    let first = unsafe { (data as *const u8).read_volatile() };
                    let second = unsafe { ((data + 1) as *const u8).read_volatile() };
                    format = if first == b'a' && second == b'8' {
                        PixelFormat::A8R8G8B8
                    } else {
                        PixelFormat::X8R8G8B8
                    };
                }
            }
            FDT_NOP => {}
            FDT_END => break,
            _ => break,
        }
    }
    None
}

#[cfg(target_arch = "riscv64")]
fn fwcfg_dma_transfer(control: u32, addr: usize, len: usize) -> bool {
    let mut dma = FwCfgDmaAccess {
        control: control.to_be(),
        length: (len as u32).to_be(),
        address: (addr as u64).to_be(),
    };
    let dma_ptr = (&mut dma as *mut FwCfgDmaAccess as u64).to_be();
    unsafe { (FW_CFG_DMA as *mut u64).write_volatile(dma_ptr) }
    let mut spins = 0usize;
    loop {
        let control = u32::from_be(unsafe { core::ptr::read_volatile(&dma.control) });
        if control == 0 {
            return true;
        }
        if (control & FW_CFG_DMA_CTL_ERROR) != 0 {
            return false;
        }
        spins += 1;
        if spins > 2_000_000 {
            return false;
        }
        core::hint::spin_loop();
    }
}

#[cfg(target_arch = "riscv64")]
fn fwcfg_write_selector(selector: u16) {
    unsafe { (FW_CFG_SELECTOR as *mut u16).write_volatile(selector.to_be()) }
}

#[cfg(target_arch = "riscv64")]
fn fwcfg_read_u8() -> u8 {
    unsafe { (FW_CFG_DATA8 as *const u8).read_volatile() }
}

#[cfg(target_arch = "riscv64")]
fn fwcfg_read_u16_be() -> u16 {
    let hi = fwcfg_read_u8() as u16;
    let lo = fwcfg_read_u8() as u16;
    (hi << 8) | lo
}

#[cfg(target_arch = "riscv64")]
fn fwcfg_read_u32_be() -> u32 {
    let b0 = fwcfg_read_u8() as u32;
    let b1 = fwcfg_read_u8() as u32;
    let b2 = fwcfg_read_u8() as u32;
    let b3 = fwcfg_read_u8() as u32;
    (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
}

#[cfg(target_arch = "riscv64")]
fn fwcfg_find_file_selector(name: &[u8]) -> Option<u16> {
    fwcfg_write_selector(FW_CFG_FILE_DIR);
    let count = fwcfg_read_u32_be() as usize;
    let mut idx = 0usize;
    while idx < count {
        let _size = fwcfg_read_u32_be();
        let selector = fwcfg_read_u16_be();
        let _reserved = fwcfg_read_u16_be();
        let mut raw_name = [0u8; 56];
        let mut i = 0usize;
        while i < raw_name.len() {
            raw_name[i] = fwcfg_read_u8();
            i += 1;
        }
        let mut matched = true;
        let mut j = 0usize;
        while j < name.len() {
            if raw_name[j] != name[j] {
                matched = false;
                break;
            }
            j += 1;
        }
        if matched {
            return Some(selector);
        }
        idx += 1;
    }
    None
}

#[cfg(target_arch = "riscv64")]
fn init_ramfb_from_fwcfg() -> Option<Framebuffer> {
    let selector = fwcfg_find_file_selector(b"etc/ramfb")?;
    let fb_addr = unsafe { core::ptr::addr_of_mut!(RAMFB_BUFFER.0) as usize };
    let cfg = RamfbCfg {
        addr: (fb_addr as u64).to_be(),
        fourcc: DRM_FORMAT_XRGB8888.to_be(),
        flags: 0u32.to_be(),
        width: (RAMFB_WIDTH as u32).to_be(),
        height: (RAMFB_HEIGHT as u32).to_be(),
        stride: (RAMFB_STRIDE as u32).to_be(),
    };
    let cfg_addr = (&cfg as *const RamfbCfg) as usize;
    let cfg_len = core::mem::size_of::<RamfbCfg>();
    let control = |sel: u16| ((sel as u32) << 16) | FW_CFG_DMA_CTL_SELECT | FW_CFG_DMA_CTL_WRITE;
    let ok_primary = fwcfg_dma_transfer(control(selector), cfg_addr, cfg_len);
    let swapped = selector.swap_bytes();
    let ok_swapped = if swapped != selector {
        fwcfg_dma_transfer(control(swapped), cfg_addr, cfg_len)
    } else {
        false
    };
    if !ok_primary && !ok_swapped {
        return None;
    }
    Some(Framebuffer {
        addr: fb_addr,
        width: RAMFB_WIDTH,
        height: RAMFB_HEIGHT,
        stride_px: RAMFB_WIDTH,
        format: PixelFormat::X8R8G8B8,
    })
}

#[cfg(target_arch = "riscv64")]
fn transform(p: Point, x0: i32, y0: i32, size: i32) -> Point {
    Point {
        x: x0 + p.x * size / 100,
        y: y0 + p.y * size / 100,
    }
}

#[cfg(target_arch = "riscv64")]
fn edge(a: Point, b: Point, p: Point) -> i32 {
    (p.x - a.x) * (b.y - a.y) - (p.y - a.y) * (b.x - a.x)
}

#[cfg(target_arch = "riscv64")]
fn fill_triangle(fb: Framebuffer, p0: Point, p1: Point, p2: Point, color: u32) {
    let mut min_x = p0.x.min(p1.x).min(p2.x);
    let mut max_x = p0.x.max(p1.x).max(p2.x);
    let mut min_y = p0.y.min(p1.y).min(p2.y);
    let mut max_y = p0.y.max(p1.y).max(p2.y);
    min_x = min_x.max(0);
    min_y = min_y.max(0);
    max_x = max_x.min((fb.width.saturating_sub(1)) as i32);
    max_y = max_y.min((fb.height.saturating_sub(1)) as i32);
    let area = edge(p0, p1, p2);
    if area == 0 {
        return;
    }
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = Point { x, y };
            let w0 = edge(p1, p2, p);
            let w1 = edge(p2, p0, p);
            let w2 = edge(p0, p1, p);
            if (area > 0 && w0 >= 0 && w1 >= 0 && w2 >= 0)
                || (area < 0 && w0 <= 0 && w1 <= 0 && w2 <= 0)
            {
                fb.put_pixel(x as usize, y as usize, color);
            }
        }
    }
}

#[cfg(target_arch = "riscv64")]
fn draw_piece(fb: Framebuffer, piece: Piece, x0: i32, y0: i32, size: i32) {
    let p0 = transform(piece.points[0], x0, y0, size);
    let p1 = transform(piece.points[1], x0, y0, size);
    let p2 = transform(piece.points[2], x0, y0, size);
    fill_triangle(fb, p0, p1, p2, piece.color);
    if piece.vertices == 4 {
        let p3 = transform(piece.points[3], x0, y0, size);
        fill_triangle(fb, p0, p2, p3, piece.color);
    }
}

#[cfg(target_arch = "riscv64")]
fn draw_letter(fb: Framebuffer, pieces: &[Piece; 7], x0: i32, y0: i32, size: i32) {
    for piece in pieces {
        draw_piece(fb, *piece, x0, y0, size);
    }
}

#[cfg(target_arch = "riscv64")]
extern "C" fn rust_main(_hartid: usize, dtb_pa: usize) -> ! {
    let framebuffer = if dtb_pa == 0 {
        parse_simple_framebuffer(dtb_pa)
    } else {
        init_ramfb_from_fwcfg()
    };
    if let Some(fb) = framebuffer {
        fb.clear(0x111827);
        let half_w = (fb.width / 2) as i32;
        let h = fb.height as i32;
        let margin = (h / 10).max(20);
        let size = (half_w - margin * 2).min(h - margin * 2).max(120);
        let o_x = (half_w - size) / 2;
        let s_x = half_w + (half_w - size) / 2;
        let y = (h - size) / 2;
        draw_letter(fb, &O_PIECES, o_x, y, size);
        draw_letter(fb, &S_PIECES, s_x, y, size);
        loop {
            core::hint::spin_loop();
        }
    }
    for c in b"framebuffer not found\n" {
        console_putchar(*c);
    }
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    shutdown(true)
}

#[cfg(not(target_arch = "riscv64"))]
mod stub {
    #[unsafe(no_mangle)]
    pub extern "C" fn main() -> i32 {
        0
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn __libc_start_main() -> i32 {
        0
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn rust_eh_personality() {}
}

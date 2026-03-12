#![no_std]

use core::arch::asm;
#[inline(always)]

pub fn print_u64(mut val: u64) {
    if val == 0 {
        print("0");
        return;
    }
    let mut buffer = [0u8; 20];
    let mut i = 20;
    while val > 0 {
        i -= 1;
        buffer[i] = (val % 10) as u8 + b'0';
        val /= 10;
    }
    unsafe {
        let s = core::str::from_utf8_unchecked(&buffer[i..]);
        print(s);
    }
}
#[inline(always)]

pub fn print(msg: &str) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 1,
            in("rdi") msg.as_ptr(),
            in("rsi") msg.len(),
            lateout("rax") _,
        );
    }
}
#[inline(always)]

pub fn exit(code: i32) -> ! {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 0,
            in("rdi") code,
            options(noreturn)
        );
    }
}
#[inline(always)]

pub fn clear() {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 2,
            lateout("rax") _,
        );
    }
}
#[inline(always)]

pub fn shutdown() {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 3,
            lateout("rax") _,
        );
    }
}
#[inline(always)]

pub fn reboot() {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 4,
            lateout("rax") _,
        );
    }
}
#[inline(always)]

pub fn read_dir(path: &str, index: usize, out_buf: &mut [u8]) -> i32 {
    let res: i32;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 5,
            in("rdi") path.as_ptr(),
            in("rsi") path.len(),
            in("rdx") index,
            in("r10") out_buf.as_mut_ptr(),
            lateout("rax") res,
        );
    }
    res
}
#[inline(always)]

pub fn set_color(color: u8) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 6,
            in("rdi") color as u64,
            lateout("rax") _,
        );
    }
}

#[repr(C)]

pub struct MomStats {
    pub active_tasks: u64,
    pub uptime_ticks: u64,
    pub kernel_used: u64,
    pub cradle_used: u64,
    pub playpen_used: u64,
    pub total_kernel: u64,
    pub total_cradle: u64,
    pub total_playpen: u64,
    pub total_invasions: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub dropped_packets: u64,
    pub playpen_max: u64,
    pub cradle_max: u64,
}

#[inline(always)]

pub fn get_mom_stats(stats: &mut MomStats) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 7,
            in("rdi") stats as *mut MomStats,
            lateout("rax") _,
        );
    }
}

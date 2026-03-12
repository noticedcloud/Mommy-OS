#![no_std]
#![no_main]

use mommylib::{print, print_u64, get_mom_stats, MomStats, exit};

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    let mut stats = MomStats {
        active_tasks: 0,
        uptime_ticks: 0,
        kernel_used: 0,
        cradle_used: 0,
        playpen_used: 0,
        total_kernel: 0,
        total_cradle: 0,
        total_playpen: 0,
        total_invasions: 0,
        rx_packets: 0,
        tx_packets: 0,
        dropped_packets: 0,
        playpen_max: 0,
        cradle_max: 0,
    };

    get_mom_stats(&mut stats);

    print("\n--- MOMSTATS ---\n");

    print("\n[SYSTEM]\n");
    print("  Uptime Ticks: "); print_u64(stats.uptime_ticks); print("\n");
    print("  Active Tasks: "); print_u64(stats.active_tasks); print("\n");

    print("\n[MEMORY]\n");

    let page_size = 4096;
    let k_bytes = stats.kernel_used * page_size;
    let c_bytes = stats.cradle_used * page_size;
    let p_bytes = stats.playpen_used * page_size;

    let c_max_bytes = stats.cradle_max * page_size;
    let p_max_bytes = stats.playpen_max * page_size;

    let c_free = c_max_bytes - c_bytes - k_bytes;
    let p_free = p_max_bytes - p_bytes;

    print("  Shared (Playpen):  Used: "); print_u64(p_bytes / 1024); print(" KB / Free: "); print_u64(p_free / 1024); print(" KB (Total: "); print_u64(p_max_bytes / 1024); print(" KB)\n");
    print("  Private (Cradle):  Used: "); print_u64(c_bytes / 1024); print(" KB / Free: "); print_u64(c_free / 1024); print(" KB (Total: "); print_u64(c_max_bytes / 1024); print(" KB)\n");
    print("  Kernel Overhead:   "); print_u64(k_bytes / 1024); print(" KB\n");

    print("\n[STATS]\n");
    print("  Allocations: Kernel: "); print_u64(stats.kernel_used); print("/"); print_u64(stats.total_kernel);
    print(" | Cradle: "); print_u64(stats.cradle_used); print("/"); print_u64(stats.total_cradle);
    print(" | Playpen: "); print_u64(stats.playpen_used); print("/"); print_u64(stats.total_playpen); print("\n");
    print("  Invasions:   "); print_u64(stats.total_invasions); print("\n");

    print("\n[NETWORK]\n");
    print("  RX Packets:   "); print_u64(stats.rx_packets); print("\n");
    print("  TX Packets:   "); print_u64(stats.tx_packets); print("\n");
    print("  Dropped:      "); print_u64(stats.dropped_packets); print("\n");

    print("----------------\n");
    print("DEBUG: All printed\n");

    exit(0);
}

#[panic_handler]

fn panic(info: &core::panic::PanicInfo) -> ! {
    print("MOMSTATS PANIC!\n");
    if let Some(location) = info.location() {
        print("At: ");
        print(location.file());
        print(":");
        print_u64(location.line() as u64);
        print("\n");
    }
    exit(1);
}

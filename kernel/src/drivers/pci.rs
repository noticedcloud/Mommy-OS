use crate::vga::print;
use core::arch::asm;
const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;
pub unsafe fn pci_read(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address = (1u32 << 31)
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC);
    outl(CONFIG_ADDRESS, address);
    inl(CONFIG_DATA)
}
pub unsafe fn pci_write(bus: u8, slot: u8, func: u8, offset: u8, value: u32) {
    let address = (1u32 << 31)
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC);
    outl(CONFIG_ADDRESS, address);
    outl(CONFIG_DATA, value);
}
unsafe fn outl(port: u16, val: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") val, options(nomem, nostack, preserves_flags));
}
unsafe fn inl(port: u16) -> u32 {
    let val: u32;
    asm!("in eax, dx", out("eax") val, in("dx") port, options(nomem, nostack, preserves_flags));
    val
}
pub unsafe fn check_device(bus: u8, device: u8) {
    let vendor_id = pci_read(bus, device, 0, 0) & 0xFFFF;
    if vendor_id == 0xFFFF {
        return;
    }
    check_function(bus, device, 0);
    let header_type = (pci_read(bus, device, 0, 0x0C) >> 16) & 0xFF;
    if (header_type & 0x80) != 0 {
        for func in 1..8 {
            let v_id = pci_read(bus, device, func, 0) & 0xFFFF;
            if v_id != 0xFFFF {
                check_function(bus, device, func);
            }
        }
    }
}
pub unsafe fn check_function(bus: u8, device: u8, func: u8) {
    let dev_vendor = pci_read(bus, device, func, 0);
    let vendor_id = dev_vendor & 0xFFFF;
    let device_id = (dev_vendor >> 16) & 0xFFFF;
    let class_rev = pci_read(bus, device, func, 0x08);
    let class_code = (class_rev >> 24) & 0xFF;
    let subclass = (class_rev >> 16) & 0xFF;
    print(b"PCI [");
    crate::vga::print_u64_vga(bus as u64);
    print(b":");
    crate::vga::print_u64_vga(device as u64);
    print(b":");
    crate::vga::print_u64_vga(func as u64);
    print(b"] ID ");
    crate::serial::print_hex(vendor_id as u64);
    print(b":");
    crate::serial::print_hex(device_id as u64);
    print(b" Class ");
    crate::serial::print_hex(class_code as u64);
    print(b":");
    crate::serial::print_hex(subclass as u64);
    print(b" -> ");
    print_device_name(
        vendor_id as u16,
        device_id as u16,
        class_code as u8,
        subclass as u8,
    );
    print(b"\n");
}
fn print_device_name(vendor: u16, device: u16, class: u8, subclass: u8) {
    if vendor == 0x8086 {
        print(b"Intel ");
        if device == 0x1237 {
            print(b"440FX Host Bridge");
            return;
        }
        if device == 0x7000 {
            print(b"PIIX3 ISA Adapter");
            return;
        }
        if device == 0x7010 {
            print(b"PIIX3 IDE Interface");
            return;
        }
        if device == 0x7111 {
            print(b"PIIX3 IDE");
            return;
        }
        if device == 0x7113 {
            print(b"PIIX4 ACPI");
            return;
        }
        if device == 0x100E {
            print(b"e1000 Ethernet");
            return;
        }
        if device == 0x1234 {
            print(b"VGA Compatible");
            return;
        }
    }
    if vendor == 0x1234 && device == 0x1111 {
        print(b"QEMU VGA");
        return;
    }
    if class == 0x03 {
        print(b"VGA Controller");
        return;
    }
    if class == 0x02 {
        print(b"Network Controller");
        return;
    }
    if class == 0x01 {
        if subclass == 0x01 {
            print(b"IDE Interface");
            return;
        }
        print(b"Mass Storage");
        return;
    }
    if class == 0x06 {
        print(b"Bridge Device");
        return;
    }
    print(b"Unknown Device");
}
pub unsafe fn get_bar0(bus: u8, slot: u8, func: u8) -> u32 {
    let header_type = (pci_read(bus, slot, func, 0x0C) >> 16) & 0xFF;
    let bar0 = pci_read(bus, slot, func, 0x10);
    if (bar0 & 1) == 0 {
        return bar0 & 0xFFFFFFF0;
    }
    return 0;
}
pub unsafe fn lspci() {
    print(b"Scanning PCI Bus...\n");
    for bus in 0..256 {
        check_device(bus as u8, 0);
    }
    for bus in 0..1 {
        for slot in 0..32 {
            check_device(bus as u8, slot as u8);
        }
    }
}

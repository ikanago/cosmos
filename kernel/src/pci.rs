use core::fmt::Display;

use arrayvec::ArrayVec;

use crate::x86::{in32, out32};

// Addresses of registers in IO address space which is used for PCI configuration space.
// To access a certain PCI configuration space,
// 1. Set an address of targeting PCI configuration space to `CONFIG_ADDRESS`
// 2. Then read/write data by using `in/out` instruction through `CONFIG_DATA`
const CONFIG_ADDRESS: u16 = 0xcf8;
const CONFIG_DATA: u16 = 0xcfc;

fn write_pci_address(address: u32) {
    out32(CONFIG_ADDRESS, address);
}

fn write_pci_data(data: u32) {
    out32(CONFIG_DATA, data);
}

fn read_pci_data() -> u32 {
    in32(CONFIG_DATA)
}

#[derive(Clone, Copy)]
pub struct PciConfig {
    bus: u8,
    device: u8,
    function: u8,
}

impl PciConfig {
    /// Make an address in the PCI configuration space.
    /// `register_offset` is an offset of the space in 4byte units.
    fn make_address(&self, register_offset: u8) -> u32 {
        let register_address = register_offset << 2;
        (1 << 31)
            | ((self.bus as u32) << 16)
            | ((self.device as u32) << 11)
            | ((self.function as u32) << 8)
            | ((register_address as u32) & 0xfc)
    }

    fn read_vendor_id(&self) -> u16 {
        write_pci_address(self.make_address(0));
        (read_pci_data() & 0xffff) as u16
    }

    fn read_device_id(&self) -> u16 {
        write_pci_address(self.make_address(0));
        ((read_pci_data() & 0xffff0000) >> 16) as u16
    }

    /// Read Base Class, Sub Class and Interface.
    fn read_class_code(&self) -> ClassCode {
        write_pci_address(self.make_address(2));
        let class_code = read_pci_data();
        let base = ((class_code & 0xff_00_00_00) >> 24) as u8;
        let sub = ((class_code & 0x00_ff_00_00) >> 16) as u8;
        let interface = ((class_code & 0x00_00_ff_00) >> 8) as u8;
        ClassCode {
            base,
            sub,
            interface,
        }
    }

    fn read_header_type(&self) -> u8 {
        write_pci_address(self.make_address(3));
        ((read_pci_data() & 0x00_ff_00_00) >> 16) as u8
    }

    fn read_bus_number(&self) -> u32 {
        write_pci_address(self.make_address(6));
        read_pci_data()
    }

    fn is_invalid_vendor_id(&self) -> bool {
        self.read_vendor_id() == 0xffff
    }
}

struct ClassCode {
    base: u8,
    sub: u8,
    interface: u8,
}

impl Display for ClassCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:02x}.{:02x}.{:02x}",
            self.base, self.sub, self.interface
        )
    }
}

pub struct PciDevice {
    bus: u8,
    device: u8,
    function: u8,
    vendor_id: u16,
    class_code: ClassCode,
    header_type: u8,
}

impl From<PciConfig> for PciDevice {
    fn from(config: PciConfig) -> Self {
        let vendor_id = config.read_vendor_id();
        let class_code = config.read_class_code();
        let header_type = config.read_header_type();
        Self {
            bus: config.bus,
            device: config.device,
            function: config.function,
            vendor_id,
            class_code,
            header_type,
        }
    }
}

impl Display for PciDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:02x}.{:02x}.{:02x}: vend {:04x}, class {}, header {:02x}",
            self.bus, self.device, self.function, self.vendor_id, self.class_code, self.header_type
        )
    }
}

#[derive(Debug)]
pub enum PciError {
    Full,
}

impl Display for PciError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PciError::Full => {
                write!(f, "More than {} devices found", MAX_DEVICES)
            }
        }
    }
}

const MAX_DEVICES: usize = 32;
pub type Devices = ArrayVec<PciDevice, MAX_DEVICES>;

pub fn scan_all_bus() -> Result<Devices, PciError> {
    let mut devices = Devices::new();

    let host_bridge = PciConfig {
        bus: 0,
        device: 0,
        function: 0,
    };
    let header_type = host_bridge.read_header_type();
    if is_single_function_device(header_type) {
        scan_bus(host_bridge, &mut devices)?;
        return Ok(devices);
    }

    for function in 1..8 {
        let target = PciConfig {
            function,
            ..host_bridge
        };
        if target.is_invalid_vendor_id() {
            continue;
        }
        scan_bus(target, &mut devices)?;
    }

    Ok(devices)
}

fn is_single_function_device(header_type: u8) -> bool {
    (header_type & 0x80) == 0
}

fn scan_bus(config: PciConfig, devices: &mut Devices) -> Result<(), PciError> {
    for device in 0..32 {
        let target = PciConfig { device, ..config };
        if target.is_invalid_vendor_id() {
            continue;
        }
        scan_device(target, devices)?;
    }

    Ok(())
}

fn scan_device(config: PciConfig, devices: &mut Devices) -> Result<(), PciError> {
    scan_function(config, devices)?;

    if is_single_function_device(config.read_header_type()) {
        return Ok(());
    }

    for function in 1..8 {
        let target = PciConfig { function, ..config };
        if target.is_invalid_vendor_id() {
            continue;
        }
        scan_function(target, devices)?;
    }

    Ok(())
}

fn scan_function(config: PciConfig, devices: &mut Devices) -> Result<(), PciError> {
    add_device(config, devices)?;

    let ClassCode { base, sub, .. } = config.read_class_code();
    if base == 0x06 && sub == 0x04 {
        // PCI-PCI bridge
        let bus_number = config.read_bus_number();
        let secondary_bus = ((bus_number >> 8) & 0xff) as u8;
        return scan_bus(
            PciConfig {
                bus: secondary_bus,
                device: 0,
                function: 0,
            },
            devices,
        );
    }
    Ok(())
}

fn add_device(config: PciConfig, devices: &mut Devices) -> Result<(), PciError> {
    let device = config.into();
    if let Err(_) = devices.try_push(device) {
        return Err(PciError::Full);
    }
    Ok(())
}

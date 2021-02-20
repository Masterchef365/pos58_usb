use std::io::{self, Error, ErrorKind, Write};

const VENDOR_ID: u16 = 0x0416;
const PRODUCT_ID: u16 = 0x5011;
pub const PAPER_WIDTH_MM: u32 = 58; 
pub const PRINTABLE_WIDTH_MM: u32 = 48; 
pub const DOTS_PER_MM: u32 = 8; 

/// A POS58 printer connected to USB, exposing Write functionality
pub struct POS58USB<'a> {
    handle: libusb::DeviceHandle<'a>,
    timeout: std::time::Duration,
    chunk_size: usize,
    endpoint_addr: u8,
}

impl<'a> POS58USB<'a> {
    /// Create a new POS58 USB instance from `context`.
    pub fn new(
        context: &'a mut libusb::Context,
        timeout: std::time::Duration,
    ) -> libusb::Result<Self> {
        let (device, device_desc, mut handle) =
            Self::get_device(context).ok_or(libusb::Error::NoDevice)?;
        let (endpoint_addr, interface_addr, packet_size) =
            Self::find_writeable_endpoint(&device, &device_desc).ok_or(libusb::Error::NotFound)?;
        handle.claim_interface(interface_addr)?;
        Ok(POS58USB {
            chunk_size: packet_size as _,
            endpoint_addr,
            handle,
            timeout,
        })
    }

    fn get_device(
        context: &mut libusb::Context,
    ) -> Option<(
        libusb::Device,
        libusb::DeviceDescriptor,
        libusb::DeviceHandle,
    )> {
        let devices = context.devices().ok()?;

        for device in devices.iter() {
            if let Ok(device_desc) = device.device_descriptor() {
                if device_desc.vendor_id() == VENDOR_ID && device_desc.product_id() == PRODUCT_ID {
                    if let Ok(handle) = device.open() {
                        return Some((device, device_desc, handle));
                    }
                }
            }
        }
        None
    }

    fn find_writeable_endpoint(
        device: &libusb::Device,
        device_desc: &libusb::DeviceDescriptor,
    ) -> Option<(u8, u8, u16)> {
        for n in 0..device_desc.num_configurations() {
            let config_desc = match device.config_descriptor(n) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for interface in config_desc.interfaces() {
                for interface_desc in interface.descriptors() {
                    for endpoint_desc in interface_desc.endpoint_descriptors() {
                        if endpoint_desc.direction() == libusb::Direction::Out
                            && endpoint_desc.transfer_type() == libusb::TransferType::Bulk
                        {
                            return Some((
                                endpoint_desc.address(),
                                interface_desc.interface_number(),
                                endpoint_desc.max_packet_size(),
                            ));
                        }
                    }
                }
            }
        }
        None
    }
}

fn translate_error(e: libusb::Error) -> Error {
    match e {
        libusb::Error::NoDevice => Error::from(ErrorKind::NotConnected),
        libusb::Error::Busy => Error::from(ErrorKind::WouldBlock),
        libusb::Error::Timeout => Error::from(ErrorKind::TimedOut),
        libusb::Error::Io => Error::from(ErrorKind::Interrupted),
        _ => Error::from(ErrorKind::Other),
    }
}

impl Write for POS58USB<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut n_written = 0;
        for chunk in buf.chunks(self.chunk_size) {
            match self
                .handle
                .write_bulk(self.endpoint_addr, chunk, self.timeout)
                {
                    Ok(bytes) => n_written += bytes,
                    Err(e) => Err(translate_error(e))?,
                }
        }
        Ok(n_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.write(b"\0").map(drop)
    }
}

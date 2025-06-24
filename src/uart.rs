use volatile_register::RW;

const LSR_DATA_READY: u8 = 0x01;
const LSR_THR_EMPTY: u8 = 0x20;

unsafe extern "C" {
    #[link_name = "_uart"]
    pub safe static mut UART: Uart;
}

#[repr(C)]
pub struct Uart {
    data: RW<u8>,
    ier: RW<u8>,
    iirfcr: RW<u8>,
    lcr: RW<u8>,
    mcr: RW<u8>,
    lsr: RW<u8>,
    spr: RW<u8>,
}

impl Uart {
    fn read_byte(&self) -> u8 {
        while self.lsr.read() & LSR_DATA_READY == 0 {}
        self.data.read()
    }

    fn put_byte(&self, byte: u8) {
        while self.lsr.read() & LSR_THR_EMPTY == 0 {}
        unsafe {
            self.data.write(byte);
        }
    }
}

impl core::fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.put_byte(c as u8);
        }
        Ok(())
    }
}

pub fn serial_put_byte(byte: u8) {
    unsafe { UART.put_byte(byte) }
}

pub fn serial_read_byte() -> u8 {
    unsafe { UART.read_byte() }
}

#[macro_export]
macro_rules! kprint {
    ($($args:tt)*) => {{
        use core::fmt::Write;
        unsafe {$crate::uart::UART.write_fmt(format_args!($($args)*)).unwrap();}
    }};
}

#[macro_export]
macro_rules! kprintln {
    () => {{
        use core::fmt::Write;
    }};
}

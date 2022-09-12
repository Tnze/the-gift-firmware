use super::at::Device;
use core::fmt::*;

pub struct CoAPClient<'a, D: Device> {
    device: &'a mut D,
}

impl<'a, D: Device> CoAPClient<'a, D> {
    pub fn new(device: &'a mut D) -> Self {
        device.write_cmd(format_args!("AT+COAPCREATE={}", 5683));
        Self { device }
    }
}

impl<'a, D: Device> Drop for CoAPClient<'a, D> {
    fn drop(&mut self) {
        let _ = self
            .device
            .write_cmd(format_args!("AT+COAPDEL"))
            .and_then(|_| self.device.read_ok());
    }
}

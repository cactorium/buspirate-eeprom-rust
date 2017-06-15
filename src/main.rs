extern crate serial;
#[macro_use]
extern crate lazy_static;

pub mod data;
mod delays;
pub mod bitbang;
pub mod onewire;
pub mod i2c;

use serial::SerialPort;

use data::ORDERED1024;
use bitbang::*;
use onewire::*;

fn do_ds2431(mut port: &mut BitBangMode) {
    let mut port = Mode::onewire(port);
    let devices = port.find_devices();
    println!("found devices {:?}", &devices);
    let dev_addr = devices.first().unwrap();
    println!("using device {:?}", &dev_addr);
    port.write_eeprom(&dev_addr, ORDERED1024);
    let read_result = port.read_eeprom(dev_addr, ORDERED1024.len());
    if read_result.as_slice() != ORDERED1024 {
        panic!("memory doesn't match; write failed!");
    }
}

fn main() {
    let mut port = serial::open("/dev/ttyUSB0").unwrap();
    port.reconfigure(&|settings| {
        try!(settings.set_baud_rate(serial::Baud115200));
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    }).unwrap();

    let mut bitbang = enter_bitbang_mode(port);
    do_ds2431(&mut bitbang);
}

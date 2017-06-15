use std::io::{Read, Write};
use std::marker::PhantomData;

use super::bitbang::*;
use super::delays::*;

pub struct I2C;

pub enum Ack {
    Ack,
    Nack,
}

impl <'a> Mode<'a, I2C> {
    pub fn i2c(port: &'a mut BitBangMode) -> Mode<'a, I2C> {
        let mut buf = [0u8; 16];

        port.write(&[0x02]).unwrap();
        sleep(RESP_DELAY_SHORT.clone());
        
        let rlen = port.read(&mut buf).unwrap();
        println!("i2c mode try, resp {:?}", &buf[0..rlen]);

        if &buf[0..rlen] != b"I2C1" {
            panic!("i2c mode entry failed!");
        }
        println!("i2c mode entered!");
        Mode { port: port, _ty: PhantomData }
    }

    fn send_byte(&mut self, b: u8) {
        let mut buf = [0u8; 16];
        self.port.write(&[b]).unwrap();
        let rlen = self.port.read(&mut buf).unwrap();
        if rlen != 1 || buf[0] != 0x01 {
            panic!("start bit send failed!");
        }
    }

    pub fn start_bit(&mut self) {
        self.send_byte(0x2);
    }
    pub fn stop_bit(&mut self) {
        self.send_byte(0x3);
    }
    pub fn ack(&mut self) {
        self.send_byte(0x6);
    }
    pub fn nack(&mut self) {
        self.send_byte(0x7);
    }
    pub fn configure(&mut self, config: Config<I2C>) {
        self.send_byte(0x40 | (0x0f & config.to_byte()));
    }

    pub fn bulk_write(&mut self, to_write: &[u8]) -> Vec<Ack> {
        flush_buffer(&mut self.port);
        let mut buf = [0u8; 17];
        let mut acks = Vec::new();
        for chunk in to_write.chunks(16) {
            assert!(chunk.len() <= 16);
            let wlen = chunk.len() + 1;

            buf[0] = 0x10 | ((chunk.len() as u8) - 1);
            buf[1..wlen].clone_from_slice(&chunk);

            self.port.write(&buf[..wlen]).unwrap();
            let mut rbuf = [0u8; 17];
            let rlen = self.port.read(&mut rbuf).unwrap();
            if rlen != wlen && !rbuf.iter().all(|x| *x == 0x01 || *x == 0x00) {
                panic!("invalid resp from i2c bulk write {:?} resp {:?}", &buf[0..wlen], &rbuf[0..rlen]);
            }
            acks.extend(rbuf[1..rlen].iter().map(|x| if *x == 0x00 { Ack::Nack } else { Ack::Ack }));
        }
        acks
    }

    pub fn read_byte(&mut self) -> u8 {
        flush_buffer(&mut self.port);
        let mut buf = [0u8; 16];
        self.port.write(&[0x4]).unwrap();
        let rlen = self.port.read(&mut buf).unwrap();
        if rlen != 1 {
            panic!("too many bytes back from i2c read: {:?}", &buf[0..rlen]);
        }
        buf[0]
    }

    pub fn send_cmd_then_read(&mut self, to_write: &[u8], read_len: usize) -> Vec<u8> {
        unimplemented!()
    }
}



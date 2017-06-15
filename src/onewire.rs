use std::io::{Read, Write};
use std::marker::PhantomData;

use super::delays::*;
use super::bitbang::*;

const ONEWIRE_PAGE_SIZE: usize = 8;
pub struct OneWire;

/// NOTE: DOESN'T WORK
impl <'a> Mode<'a, OneWire> {
    pub fn onewire(port: &'a mut BitBangMode) -> Mode<'a, OneWire> {
        let mut buf = [0u8; 16];

        port.write(&[0x04]).unwrap();
        sleep(RESP_DELAY_SHORT.clone());
        
        let rlen = port.read(&mut buf).unwrap();
        println!("1wire mode try, resp {:?}", &buf[0..rlen]);

        if &buf[0..rlen] != b"1W01" {
            panic!("1wire mode entry failed!");
        }
        println!("1write mode entered!");

        println!("1wire power on...");
        port.write(&[0x48]).unwrap();
        sleep(RESP_DELAY_SHORT.clone());
        let rlen = port.read(&mut buf).unwrap();
        println!("1wire power on resp {:?}", &buf[0..rlen]);
        if rlen != 1 || buf[0] != 0x01 {
            panic!("invalid response");
        }

        let mut owm = Mode { port: port, _ty: PhantomData };

        sleep(RESP_DELAY_REALLY_LONG.clone());

        owm.reset();
        owm
    }

    pub fn reset(&mut self) {
        let mut buf = [0u8; 16];
        println!("resetting one wire bus...");
        self.port.write(&[0x02]).unwrap();
        sleep(RESP_DELAY_SHORT.clone());
        let rlen = self.port.read(&mut buf).unwrap();
        println!("1wire mode bus reset, resp {:?}", &buf[0..rlen]);

        if rlen != 1 || buf[0] != 0x01 {
            panic!("one wire reset failed");
        }
    }

    pub fn find_devices(&mut self) -> Vec<DeviceAddress> {
        println!("looking for devices..");
        let mut buf = [0u8; 8];
        let mut ret = vec![];
        self.port.write(&[0x08]).unwrap();
        sleep(RESP_DELAY_LONG.clone());
        let rlen = self.port.read(&mut buf[0..1]).unwrap();
        if rlen != 1 || buf[0] != 0x01 {
            panic!("invalid response");
        }

        sleep(RESP_DELAY_LONG.clone());
        let mut rlen = self.port.read(&mut buf).unwrap();
        println!("resp: {:?}", &buf[0..rlen]);
        while rlen == 8 && &buf != &[0xff; 8] {
            ret.push(buf.to_vec());
            rlen = self.port.read(&mut buf).unwrap();
            println!("resp: {:?}", &buf[0..rlen]);
        }
        ret
    }

    pub fn raw_write(&mut self, msg: &[u8]) {
        println!("sending 1wire write {:?}", msg);
        // prepare a set of bulk writes that will cover the entire len
        let total_len = msg.len();
        let mut len_sent = 0;
        let mut byte_iter = msg.iter();

        flush_buffer(self.port);

        while len_sent < total_len {
            let bulk_len = if (total_len - len_sent) > 4 { 4 } else { total_len - len_sent };
            let write_header = 0x10 | ((bulk_len - 1) as u8);
            let mut write_msg = Vec::new();
            write_msg.push(write_header);
            for _ in 0..bulk_len {
                write_msg.push(*byte_iter.next().unwrap());
            }
            self.port.write(&write_msg).unwrap();
            sleep(RESP_DELAY_LONG.clone());
            let mut buf = [0u8; 17];
            let rlen = self.port.read(&mut buf).unwrap();
            println!("bulk write len {} msg {:?} resp {:?}", bulk_len, &write_msg, &buf[0..rlen]);
            if rlen != (bulk_len + 1) || !buf[0..rlen].iter().all(|x| *x == 0x01) {
                panic!("bulk write failed!");
            }

            len_sent += bulk_len;
        }
    }

    pub fn bulk_read(&mut self, sz: usize) -> Vec<u8> {
        let mut ret = Vec::new();
        for _ in 0..sz {
            self.port.write(&[0x4]).unwrap();
            sleep(RESP_DELAY_SHORT.clone());
            let mut buf = [0u8];
            let rlen = self.port.read(&mut buf).unwrap();
            if rlen != 1 {
                panic!("bulk read failed!");
            }
            ret.push(buf[0]);
        }
        ret
    }

    pub fn select(&mut self, addr: &[u8]) {
        let mut select_msg = vec![0x55];
        select_msg.extend_from_slice(addr);
        self.raw_write(&select_msg);
    }

    pub fn write_eeprom(&mut self, dev_addr: &[u8], data: &[u8]) {
        let mut addr = 0u16;
        let iter = data.chunks(ONEWIRE_PAGE_SIZE);
        for chunk in iter {
            println!("writing eeprom chunk {:x} {:?}", addr, chunk);
            // write to the scratchpad
            let mut write_msg = Vec::new();
            write_msg.push(0x0f);                           // command
            write_msg.push((addr & 0xff) as u8);            // lower bits of address
            write_msg.push(((addr >> 8) & 0xff) as u8);     // higher bits of address
            write_msg.extend_from_slice(chunk);            // the data to transfer
            self.select(dev_addr);
            self.raw_write(&write_msg);
            // reset the connection for the next transaction
            sleep(RESP_DELAY_SHORT.clone());
            self.reset();
            sleep(RESP_DELAY_SHORT.clone());

            // read the scratchpad to verify that it matches what was sent,
            // and copy the access code for the copy scratchpad instruction
            let mut read_msg = Vec::new();
            read_msg.push(0xaa);
            self.select(dev_addr);
            self.raw_write(&read_msg);
            let read_results = self.bulk_read(3); // read three bytes to get the access code
            let read_scratchpad = self.bulk_read(chunk.len()); // read the scratchpad to make sure it matches the inserted chunk
            println!("read scratchpad {:?} {:?}", &read_results, &read_scratchpad);
            if read_scratchpad != chunk {
                // FIXME: retry instead of failing
                panic!("invalid chunk read");
            }
            // reset for the next transaction
            self.reset();
            sleep(RESP_DELAY_SHORT.clone());

            // copy from scratchpad to main memory
            let mut copy_msg = Vec::new();
            copy_msg.push(0x55);
            copy_msg.extend_from_slice(&read_results);
            self.select(dev_addr);
            self.raw_write(&copy_msg);
            // reset for the next transaction
            self.reset();
            sleep(RESP_DELAY_SHORT.clone());

            addr += chunk.len() as u16;
        }
    }

    pub fn read_eeprom(&mut self, dev_addr: &[u8], len: usize) -> Vec<u8> {
        self.select(dev_addr);
        let ret = self.bulk_read(len);
        self.reset();
        ret
    }
}



//! Host-side register/bus checks, without a radio. Run with rustc --test.
//! These verify the port's command sequence, not analog RF performance.
#![allow(dead_code)]
extern crate self as at32f421_pac;
extern crate self as cortex_m;

pub struct Gpioa;
pub struct Gpioc;
pub mod peripheral {
    pub struct SYST;
}
pub mod hal {
    pub mod delay {
        pub fn us(_: &mut crate::peripheral::SYST, _: u32) {}
        pub fn ms(_: &mut crate::peripheral::SYST, _: u32) {}
    }
}

mod board {
    use crate::{Gpioa, Gpioc};
    use std::cell::RefCell;
    struct Bus {
        regs: [u16; 128],
        bits: Vec<bool>,
        data: bool,
        read_bit: usize,
        writes: Vec<(u8, u16)>,
    }
    impl Default for Bus {
        fn default() -> Self {
            Self {
                regs: [0; 128],
                bits: vec![],
                data: false,
                read_bit: 0,
                writes: vec![],
            }
        }
    }
    thread_local! { static BUS: RefCell<Bus> = RefCell::new(Bus::default()); }
    pub fn reset() {
        BUS.with(|b| {
            let mut b = b.borrow_mut();
            *b = Bus::default();
            b.regs[0] = 0x4819;
        });
    }
    pub fn reg(addr: usize) -> u16 {
        BUS.with(|b| b.borrow().regs[addr])
    }
    pub fn writes() -> Vec<(u8, u16)> {
        BUS.with(|b| b.borrow().writes.clone())
    }
    pub fn set_fd6818_sck(_: &Gpioa, high: bool) {
        if high {
            BUS.with(|b| {
                let mut b = b.borrow_mut();
                let data = b.data;
                b.bits.push(data);
            });
        }
    }
    pub fn set_shared_sda(_: &Gpioa, high: bool) {
        BUS.with(|b| b.borrow_mut().data = high);
    }
    fn word(bits: &[bool]) -> u16 {
        bits.iter().fold(0, |v, b| (v << 1) | u16::from(*b))
    }
    pub fn set_fd6818_scn(_: &Gpioc, high: bool) {
        BUS.with(|bus| {
            let mut b = bus.borrow_mut();
            if !high {
                b.bits.clear();
                b.read_bit = 0;
            } else if b.bits.len() == 24 && !b.bits[0] {
                let addr = word(&b.bits[..8]) as u8;
                let value = word(&b.bits[8..]);
                b.regs[addr as usize] = value;
                b.writes.push((addr, value));
            }
        });
    }
    pub fn read_shared_sda(_: &Gpioa) -> bool {
        BUS.with(|bus| {
            let mut b = bus.borrow_mut();
            let addr = (word(&b.bits[..8]) & 0x7f) as usize;
            let value = b.regs[addr] & (1 << (15 - b.read_bit)) != 0;
            b.read_bit += 1;
            value
        })
    }
    pub fn set_shared_sda_input(_: &Gpioa) {}
    pub fn set_shared_sda_output(_: &Gpioa) {}
}

#[path = "../src/drivers/bk4819.rs"]
mod driver;

#[test]
fn bk_init_uses_4819_recipe_and_keeps_pa_off() {
    board::reset();
    let mut syst = peripheral::SYST;
    let mut rf = driver::Fd6818::new(&Gpioa, &Gpioc);
    assert_eq!(rf.chip_id(&mut syst), driver::EXPECTED_CHIP_ID);
    rf.init(&mut syst);
    assert_eq!(board::reg(0x37), 0x1d0f);
    assert_eq!(board::reg(0x13), 0x03be);
    assert_eq!(board::reg(0x1f), 0x5454);
    assert_eq!(board::reg(0x3e), 0xa037);
    assert_eq!(board::reg(0x36) & 0x80, 0);
    assert_eq!(board::reg(0x33) & 0x0404, 0); // GPIO2 output low
    assert_eq!(board::reg(0x34), 0x0200);
    assert_eq!(board::reg(0x40) & 0x0fff, 0x04d2);
    assert_eq!(board::reg(0x7d), 0xe952);
    rf.set_scramble(&mut syst, 0);
    assert_eq!(board::reg(0x40) & 0x0fff, 0x04e0);
    assert!(!board::writes().contains(&(0x37, 0x9d1f)));
    assert!(!board::writes().iter().any(|&(r, _)| r == 0x28));
}

#[test]
fn band_switches_follow_5rh_board_rather_than_k6_board() {
    board::reset();
    let mut syst = peripheral::SYST;
    let mut rf = driver::Fd6818::new(&Gpioa, &Gpioc);
    rf.set_band_vhf(&mut syst);
    assert_eq!(board::reg(0x33) & 0x60, 0x20);
    rf.set_band_uhf(&mut syst);
    assert_eq!(board::reg(0x33) & 0x60, 0x40);
    rf.set_tx_band(&mut syst);
    assert_eq!(board::reg(0x33) & 0x18, 0x10);
    rf.rf_off(&mut syst);
    assert_eq!(board::reg(0x33) & 0x78, 0);
    assert_eq!(board::reg(0x36) & 0x80, 0);
}

#[test]
fn calibrated_pa_target_keeps_stock_bk_gain_words_and_boundaries() {
    board::reset();
    let mut syst = peripheral::SYST;
    let mut rf = driver::Fd6818::new(&Gpioa, &Gpioc);
    rf.set_pa_calibration(0x42);
    for (hz, gain) in [
        (144_000_000, 0xd7),
        (154_999_999, 0xd7),
        (155_000_000, 0xc3),
        (174_999_999, 0xc3),
        (175_000_000, 0xd7),
        (446_000_000, 0xd7),
    ] {
        rf.pa_enable_at(&mut syst, driver::Power::Low, hz);
        assert_eq!(board::reg(0x36), 0x4200 | gain, "Low at {hz}");
        for power in [driver::Power::Mid, driver::Power::High] {
            rf.pa_enable_at(&mut syst, power, hz);
            assert_eq!(board::reg(0x36), 0x42ff, "Mid/High at {hz}");
        }
    }
}

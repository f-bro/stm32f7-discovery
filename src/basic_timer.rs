use board::tim6::Tim6;
use board::rcc::Rcc;
use board::tim6::Egr;

#[derive(PartialEq)]
pub enum BasicTimers {
    Tim6,
    Tim7,
}

pub struct BasicTimer {
    tim: &'static mut Tim6,
}

impl BasicTimer {
    
    pub fn new_with_interrupt(tim: &'static mut Tim6, rcc: &mut &mut Rcc, tim_num: BasicTimers) -> BasicTimer {
        // TODO: Maybe use address of tim for if
        if tim_num == BasicTimers::Tim6 {
            rcc.apb1enr.update(|r| r.set_tim6en(true));
        } else {
            rcc.apb1enr.update(|r| r.set_tim7en(true));
        }
        tim.dier.update(|r| r.set_uie(true));
        tim.cr1.update(|r| r.set_urs(true));
        BasicTimer {
            tim: tim,
        }
    }

    pub fn set_timeout_in_ms(&mut self, ms: u16) {
        // TODO: Use u32 and calculate prescalar
        self.tim.psc.update(|r| r.set_psc(45_000));
        self.tim.arr.update(|r| r.set_arr(ms));
    }

    pub fn resume(&mut self) {
        let mut egr = Egr::default();
        egr.set_ug(true);
        self.tim.egr.write(egr);
        self.tim.cr1.update(|r| r.set_cen(true));
    }

    pub fn stop(&mut self) {
        self.tim.cr1.update(|r| r.set_cen(true));
    }

    pub fn reset(&mut self) {
        self.tim.cnt.update(|r| r.set_cnt(0));
    }

}
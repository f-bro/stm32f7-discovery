#![feature(lang_items)]
#![feature(const_fn)]
#![feature(alloc)]
#![feature(asm)]
#![feature(compiler_builtins_lib)]
#![no_std]
#![no_main]

#[macro_use]
extern crate stm32f7_discovery as stm32f7;

// initialization routines for .data and .bss

#[macro_use]
extern crate alloc;
extern crate compiler_builtins;
extern crate r0;
extern crate smoltcp;

// hardware register structs with accessor methods
use stm32f7::{board, embedded, lcd, sdram, system_clock,};


const SIN: [usize; 480] = [136,137,139,141,143,144,146,148,150,151,153,155,157,158,160,162,164,165,167,169,171,172,174,176,177,179,181,182,184,186,187,189,
191,192,194,195,197,199,200,202,203,205,206,208,209,211,212,214,215,217,218,219,221,222,224,225,226,227,229,230,231,233,234,235,
236,237,239,240,241,242,243,244,245,246,247,248,249,250,251,252,253,254,255,255,256,257,258,259,259,260,261,261,262,263,263,264,
264,265,265,266,266,267,267,268,268,268,269,269,269,270,270,270,270,270,271,271,271,271,271,271,271,271,271,271,271,271,271,270,
270,270,270,270,269,269,269,268,268,268,267,267,266,266,265,265,264,264,263,263,262,261,261,260,259,259,258,257,256,255,255,254,
253,252,251,250,249,248,247,246,245,244,243,242,241,240,239,237,236,235,234,233,231,230,229,227,226,225,224,222,221,219,218,217,
215,214,212,211,209,208,206,205,203,202,200,199,197,195,194,192,191,189,187,186,184,182,181,179,177,176,174,172,171,169,167,165,
164,162,160,158,157,155,153,151,150,148,146,144,143,141,139,137,136,134,132,130,128,127,125,123,121,120,118,116,114,113,111,109,
107,106,104,102,100,99,97,95,94,92,90,89,87,85,84,82,80,79,77,76,74,72,71,69,68,66,65,63,62,60,59,57,
56,54,53,52,50,49,47,46,45,44,42,41,40,38,37,36,35,34,32,31,30,29,28,27,26,25,24,23,22,21,20,19,
18,17,16,16,15,14,13,12,12,11,10,10,9,8,8,7,7,6,6,5,5,4,4,3,3,3,2,2,2,1,1,1,
1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,1,1,2,2,2,3,3,3,4,4,5,5,6,6,
7,7,8,8,9,10,10,11,12,12,13,14,15,16,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,34,
35,36,37,38,40,41,42,44,45,46,47,49,50,52,53,54,56,57,59,60,62,63,65,66,68,69,71,72,74,76,77,79,
80,82,84,85,87,89,90,92,94,95,97,99,100,102,104,106,107,109,111,113,114,116,118,120,121,123,125,127,128,130,132,134,];

#[no_mangle]
pub unsafe extern "C" fn reset() -> ! {
    extern "C" {
        static __DATA_LOAD: u32;
        static mut __DATA_END: u32;
        static mut __DATA_START: u32;

        static mut __BSS_START: u32;
        static mut __BSS_END: u32;
    }

    // initializes the .data section (copy the data segment initializers from flash to RAM)
    r0::init_data(&mut __DATA_START, &mut __DATA_END, &__DATA_LOAD);
    // zeroes the .bss section
    r0::zero_bss(&mut __BSS_START, &__BSS_END);

    stm32f7::heap::init();

    // enable floating point unit
    let scb = stm32f7::cortex_m::peripheral::scb_mut();
    scb.cpacr.modify(|v| v | 0b1111 << 20);
    asm!("DSB; ISB;"::::"volatile"); // pipeline flush

    main(board::hw());
}

// WORKAROUND: rust compiler will inline & reorder fp instructions into
#[inline(never)] //             reset() before the FPU is initialized
fn main(hw: board::Hardware) -> ! {
    use embedded::interfaces::gpio::{self, Gpio};

    let x = vec![1, 2, 3, 4, 5];
    assert_eq!(x.len(), 5);
    assert_eq!(x[3], 4);

    let board::Hardware {
        rcc,
        pwr,
        flash,
        fmc,
        ltdc,
        gpio_a,
        gpio_b,
        gpio_c,
        gpio_d,
        gpio_e,
        gpio_f,
        gpio_g,
        gpio_h,
        gpio_i,
        gpio_j,
        gpio_k,

        ..
    } = hw;

    let mut gpio = Gpio::new(
        gpio_a,
        gpio_b,
        gpio_c,
        gpio_d,
        gpio_e,
        gpio_f,
        gpio_g,
        gpio_h,
        gpio_i,
        gpio_j,
        gpio_k,
    );

    system_clock::init(rcc, pwr, flash);

    // enable all gpio ports
    rcc.ahb1enr.update(|r| {
        r.set_gpioaen(true);
        r.set_gpioben(true);
        r.set_gpiocen(true);
        r.set_gpioden(true);
        r.set_gpioeen(true);
        r.set_gpiofen(true);
        r.set_gpiogen(true);
        r.set_gpiohen(true);
        r.set_gpioien(true);
        r.set_gpiojen(true);
        r.set_gpioken(true);
    });

    // configure led pin as output pin
    let led_pin = (gpio::Port::PortI, gpio::Pin::Pin1);
    let mut led = gpio.to_output(
        led_pin,
        gpio::OutputType::PushPull,
        gpio::OutputSpeed::Low,
        gpio::Resistor::NoPull,
    ).expect("led pin already in use");

    // turn led on
    led.set(true);

    let button_pin = (gpio::Port::PortI, gpio::Pin::Pin11);
    let _ = gpio.to_input(button_pin, gpio::Resistor::NoPull)
        .expect("button pin already in use");

    // init sdram (needed for display buffer)
    sdram::init(rcc, fmc, &mut gpio);

    // lcd controller
    let mut lcd = lcd::init(ltdc, rcc, &mut gpio);
    //let mut layer_1 = lcd.layer_1_with_double_frame_buffer().unwrap();
    let mut layer_1 = lcd.layer_1_with_double_frame_buffer().unwrap();
    let mut layer_2 = lcd.layer_2().unwrap();

    //layer_1.clear_all();
    layer_1.clear_all();
    layer_2.clear();
    lcd::init_stdout(layer_2);

    let mut printed_points_l1 = CircularBuffer::new();
    let mut printed_points_l2 = CircularBuffer::new();
    let mut status = false;
    hprintln!("Start");
    let mut x = 0;
    loop {
        let current = system_clock::ticks();
        if status {
            for i in 0..printed_points_l1.len() {
                layer_1.print_point_color_at(i, printed_points_l1.get(i), lcd::Color::rgb(0,0,255));
            }
            printed_points_l1.clear();
        } else {
            for i in 0..printed_points_l2.len() {
                layer_1.print_point_color_at(i, printed_points_l2.get(i), lcd::Color::rgb(0,0,255));
            }
            printed_points_l2.clear();
        }

        if status {
            for i in 0..printed_points_l2.len() {
                printed_points_l1.push_back(printed_points_l2.get(i));
            }
        } else {
            for i in 0..printed_points_l1.len() {
                printed_points_l2.push_back(printed_points_l1.get(i));
            }
        }
        

        if status {
            if printed_points_l1.len() < 480 {
                printed_points_l1.push_back(SIN[x]);
                for i in 0..printed_points_l1.len() {
                    layer_1.print_point_at(i, printed_points_l1.get(i));
                }
            } else {
                printed_points_l1.pop_front();
                printed_points_l1.push_back(SIN[x % 480]);
                for i in 0..printed_points_l1.len() {
                    layer_1.print_point_at(i, printed_points_l1.get(i));
                }
            }
            
        } else {
            if printed_points_l2.len() < 480 {
                printed_points_l2.push_back(SIN[x]);
                for i in 0..printed_points_l2.len() {
                    layer_1.print_point_at(i, printed_points_l2.get(i));
                }
            } else {
                printed_points_l2.pop_front();
                printed_points_l2.push_back(SIN[x % 480]);
                for i in 0..printed_points_l2.len() {
                    layer_1.print_point_at(i, printed_points_l2.get(i));
                }
            }
            
        }

        hprintln!("ticks {}", system_clock::ticks() - current);
    
        layer_1.toggle_buffer(&mut lcd);
        x += 1;
        status = !status;
        system_clock::wait(15);
    }
}

struct CircularBuffer {
    points: [usize; 480],
    start: usize,
    end: usize,
    size: usize,
}

impl CircularBuffer {
    fn new() -> CircularBuffer {
        CircularBuffer {
            points: [0; 480],
            start: 0,
            end: 0,
            size: 0,
        }
        
    }

    fn clear(&mut self) {
        self.start = 0;
        self.end = 0;
        self.size = 0;
    } 

    fn get(&self, x: usize) -> usize {
        self.points[(self.start + x) % self.points.len()]
    }

    fn push_back(&mut self, point: usize) {
        assert!(self.len() < self.points.len());
        self.points[self.end % self.points.len()] = point;
        self.end = (self.end + 1) % self.points.len();
        self.size += 1;
    }

    fn pop_front(&mut self) -> usize {
        assert!(self.len() > 0);
        let index = self.start;
        self.start = (self.start + 1) % self.points.len();
        self.size -= 1;
        self.points[index]
    }

    fn len(&self) -> usize {
        self.size
    }
}
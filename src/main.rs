#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

extern crate alloc;

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                         // extern crate panic_abort; // requires nightly
                         // extern crate panic_itm; // logs messages over ITM; requires ITM support
                         // extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;

use embedded_graphics_core::pixelcolor::BinaryColor;
use embedded_graphics_core::prelude::OriginDimensions;
use embedded_hal::digital::v2::InputPin;
use slint::platform::software_renderer::PremultipliedRgbaColor;
use slint::platform::software_renderer::TargetPixel;
use slint::platform::Key;
use slint::platform::WindowAdapter;
use slint::platform::WindowEvent;
use slint::PhysicalSize;
use stm32f4xx_hal::i2c::Mode;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::time::Hertz;

slint::include_modules!();

#[entry]
fn main() -> ! {
    let dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();
    let cp = stm32f4xx_hal::pac::CorePeripherals::take().unwrap();

    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    let gpioa = dp.GPIOA.split();

    let _led_wifi = gpiob.pb2.into_push_pull_output();
    let _led_user = gpioc.pc13.into_push_pull_output();
    let _led_azure = gpioa.pa15.into_push_pull_output();

    let _led_r = gpiob.pb4.into_push_pull_output();
    let _led_b = gpioc.pc7.into_push_pull_output();
    let _led_g = gpiob.pb3.into_push_pull_output();

    let btn_a = gpioa.pa4.into_pull_up_input();
    let btn_b = gpioa.pa10.into_pull_up_input();

    use ssd1306::prelude::*;

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();
    let i2c = stm32f4xx_hal::i2c::I2c1::new(
        dp.I2C1,
        (gpiob.pb8, gpiob.pb9),
        Mode::Standard {
            frequency: stm32f4xx_hal::time::Hertz::kHz(400),
        },
        &clocks,
    );

    let interface = ssd1306::I2CDisplayInterface::new(i2c);
    let mut disp = ssd1306::Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    disp.init().unwrap();
    disp.flush().unwrap();

    disp.clear();

    // -------- Setup Allocator --------
    const HEAP_SIZE: usize = 200 * 1024;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    #[global_allocator]
    static ALLOCATOR: alloc_cortex_m::CortexMHeap = alloc_cortex_m::CortexMHeap::empty();
    unsafe {
        ALLOCATOR.init(
            &mut HEAP as *const u8 as usize,
            core::mem::size_of_val(&HEAP),
        )
    }

    struct MyPlatform {
        window: alloc::rc::Rc<slint::platform::software_renderer::MinimalSoftwareWindow<1>>,
        timer: stm32f4xx_hal::dwt::Instant,
        freq: Hertz,
    }

    impl slint::platform::Platform for MyPlatform {
        fn create_window_adapter(&self) -> alloc::rc::Rc<dyn slint::platform::WindowAdapter> {
            self.window.clone()
        }
        fn duration_since_start(&self) -> core::time::Duration {
            core::time::Duration::from_millis((self.timer.elapsed() / self.freq.to_kHz()) as u64)
        }
    }

    let window = slint::platform::software_renderer::MinimalSoftwareWindow::new();
    let timer = stm32f4xx_hal::dwt::MonoTimer::new(cp.DWT, cp.DCB, &clocks);
    slint::platform::set_platform(alloc::boxed::Box::new(MyPlatform {
        window: window.clone(),
        timer: timer.now(),
        freq: timer.frequency(),
    }))
    .unwrap();

    let ui = MainWindow::new();
    let s = disp.size();
    ui.window().set_size(PhysicalSize::new(s.width, s.height));
    ui.show();
    let mut line = [GrayPixel(0); 320];

    let mut btns = [
        (&btn_a as &dyn InputPin<Error = _>, Key::LeftArrow, false),
        (&btn_b as &dyn InputPin<Error = _>, Key::RightArrow, false),
    ];

    loop {
        slint::platform::update_timers_and_animations();
        window.draw_if_needed(|renderer| {
            use embedded_graphics_core::prelude::*;
            struct DisplayWrapper<'a, T>(&'a mut T, &'a mut [GrayPixel]);
            impl<T: DrawTarget<Color = BinaryColor>>
                slint::platform::software_renderer::LineBufferProvider for DisplayWrapper<'_, T>
            {
                type TargetPixel = GrayPixel;
                fn process_line(
                    &mut self,
                    line: usize,
                    range: core::ops::Range<usize>,
                    render_fn: impl FnOnce(&mut [Self::TargetPixel]),
                ) {
                    let rect = embedded_graphics_core::primitives::Rectangle::new(
                        Point::new(range.start as _, line as _),
                        Size::new(range.len() as _, 1),
                    );
                    render_fn(&mut self.1[range.clone()]);
                    self.0
                        .fill_contiguous(
                            &rect,
                            self.1[range.clone()].iter().map(|src| {
                                if src.0 > 0x88 {
                                    BinaryColor::On
                                } else {
                                    BinaryColor::Off
                                }
                            }),
                        )
                        .map_err(drop)
                        .unwrap();
                }
            }
            renderer.render_by_line(DisplayWrapper(&mut disp, line.as_mut_slice()));
            disp.flush().unwrap();
        });

        for (btn, key, pressed) in &mut btns {
            let p = btn.is_high().unwrap();
            if p && !*pressed {
                window.window().dispatch_event(WindowEvent::KeyPressed {
                    text: (*key).into(),
                })
            } else if !p && *pressed {
                window.window().dispatch_event(WindowEvent::KeyReleased {
                    text: (*key).into(),
                })
            };
            *pressed = p;
        }
    }
}

/// A 8bit grayscale pixel
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct GrayPixel(pub u8);

impl TargetPixel for GrayPixel {
    fn blend(&mut self, color: PremultipliedRgbaColor) {
        let a = (u8::MAX - color.alpha) as u16;

        let c = (color.red as u16 + color.blue as u16 + color.green as u16) / 3;

        self.0 = (((c << 8) + self.0 as u16 * a) >> 8) as u8;
    }

    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(((r as u16 + g as u16 + b as u16) / 3) as u8)
    }
}

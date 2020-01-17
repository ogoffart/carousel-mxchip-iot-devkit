#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
// extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;

use stm32f4::stm32f412;
use stm32f4xx_hal::{stm32, prelude::*};

#[entry]
fn main() -> ! {
    cortex_m::asm::delay(10000);

    let dp = stm32::Peripherals::take().unwrap();

    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    let gpioa = dp.GPIOA.split();

    let mut led_wifi = gpiob.pb2.into_push_pull_output();
    let mut led_user = gpioc.pc13.into_push_pull_output();
    let mut led_azure = gpioa.pa15.into_push_pull_output();

    let mut led_r = gpiob.pb4.into_push_pull_output();
    let mut led_b = gpioc.pc7.into_push_pull_output();
    let mut led_g = gpiob.pb3.into_push_pull_output();

    let btn_a = gpioa.pa4.into_pull_up_input();
    let btn_b = gpioa.pa10.into_pull_up_input();

    use ssd1306::prelude::*;

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();
    let i2c = stm32f4xx_hal::i2c::I2c::i2c1(dp.I2C1,
        (gpiob.pb8.into_alternate_af4(),
         gpiob.pb9.into_alternate_af4()),
        stm32f4xx_hal::time::KiloHertz(400), clocks );

    let mut disp: GraphicsMode<_> = ssd1306::Builder::new().connect_i2c(i2c).into();
    disp.init().unwrap();
    disp.flush().unwrap();

    let mut snake = snake::Snake::default();
    let mut last_touch = false;
    loop {
        let mut is_left = false;
        let mut is_right = false;

        for _ in 0..200 {
            if !last_touch {
                is_left = is_left || !btn_a.is_high().unwrap();
                is_right = is_right || !btn_b.is_high().unwrap();
            }
            if btn_a.is_high().unwrap() && btn_b.is_high().unwrap() {
                last_touch = false;
            } else {
                last_touch = true;
            }
            cortex_m::asm::delay(10000);
        }
        if is_left && is_right {
            is_left = false;
            is_right = false;
        }
        snake::advance(&mut snake, is_left, is_right);
        disp.clear();
        snake::draw(&snake, &mut disp, 128, 64);
        disp.flush().unwrap();
    };
}





mod snake {
use embedded_graphics::Drawing;

const BOARD_WIDTH : isize = 24;
const BOARD_HEIGHT : isize = 12;
const SNAKE_MAX : usize = 128;

#[derive(Default, Eq,PartialEq, Clone, Copy)]
struct Point(isize, isize);

pub struct Snake {
    tail: [Point; SNAKE_MAX],
    dx: isize,
    dy: isize,
    len: usize,
    offset: usize,
    apple : Point,
}

impl Default for Snake {
    fn default() -> Self {
        Snake{
            tail: [Point(0,0); SNAKE_MAX],
            dx: 1,
            dy: 0,
            len: 1,
            offset: 0,
            apple: Point(2,3),
        }
    }
}

pub fn advance(snake: &mut Snake, turn_left : bool, turn_right: bool) {
    if turn_left {
        let ndx = snake.dy;
        snake.dy = -snake.dx;
        snake.dx = ndx;
    } else if turn_right {
        let ndx = - snake.dy;
        snake.dy = snake.dx;
        snake.dx = ndx;
    }

    let p = snake.tail[snake.offset];
    let mut p = Point(p.0 + snake.dx, p.1 + snake.dy);

    //if p.0 < 0 || p.0 >= BOARD_WIDTH || p.1 < 0 || p.1 >= BOARD_HEIGHT {
    //   *snake =  Snake::default();
    //   return;
    //}
    p.0 = (BOARD_WIDTH + p.0) % BOARD_WIDTH;
    p.1 = (BOARD_HEIGHT + p.1) % BOARD_HEIGHT;
    for x in 0..snake.len {
        if p == snake.tail[(SNAKE_MAX + snake.offset - x) % SNAKE_MAX] {
            *snake =  Snake::default();
            return;
        }
    }

    if p == snake.apple {
        snake.len += 1;
        let r = (snake.len as isize * 7 + p.0  * 13 + p.1 * 73 + snake.offset as isize * 29
            + (snake.dy + 4) * 97 + (snake.dx + 3) * 53) * 197;
        snake.apple = Point(r % BOARD_WIDTH, (r / BOARD_WIDTH) % BOARD_HEIGHT);
        if snake.len >= SNAKE_MAX {
            snake.len = SNAKE_MAX-1
        }
    }
    snake.offset += 1;
    if snake.offset == SNAKE_MAX {
        snake.offset = 0
    }

    snake.tail[snake.offset] = p;
}

pub fn draw<C : embedded_graphics::pixelcolor::PixelColor, D : Drawing<C>>(
    snake: &Snake, disp : &mut D, screen_width: isize, screen_height: isize) {

    use embedded_graphics::prelude::*;
    use embedded_graphics::primitives::{Circle, Line, Rect};

    let sqw = (screen_width/ BOARD_WIDTH)  as i32;
    let sqh = (screen_height/ BOARD_HEIGHT)  as i32;

    //disp.draw(Rect::new(Coord::new(0, 0), Coord::new(screen_width, screen_height)).with_fill(Some(0.into())) .into_iter());
    for x in 0..snake.len {
        let p = snake.tail[(SNAKE_MAX + snake.offset - x) % SNAKE_MAX];
        disp.draw(Rect::new(Coord::new(p.0 as i32 * sqw, p.1 as i32 * sqh),
                Coord::new((p.0 +1) as i32 * sqw, (p.1 + 1) as i32 * sqh))
            .with_fill(Some(1.into())).into_iter());
    }
    disp.draw(Line::new(Coord::new(snake.apple.0 as i32 * sqw, snake.apple.1 as i32 * sqh),
            Coord::new((snake.apple.0 +1) as i32 * sqw, (snake.apple.1 + 1) as i32 * sqh) )
        .with_stroke(Some(2.into())) .into_iter());
    disp.draw(Line::new(Coord::new((snake.apple.0 + 1) as i32 * sqw, snake.apple.1 as i32 * sqh),
            Coord::new(snake.apple.0 as i32 * sqw, (snake.apple.1 + 1) as i32 * sqh) )
        .with_stroke(Some(2.into())) .into_iter());
}


}











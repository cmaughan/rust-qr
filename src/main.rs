use minifb::{Key, Window, WindowOptions};

use maths_rs::prelude::*;

type Rgb32 = u32;
type Vector2u = Vec2<u32>;

// Window output buffer
struct OutputBuffer {
    size: Vector2u,
    buffer: Vec<u32>,
}

// Read the QR code
fn read_qr() -> Vec<Vec<u8>> {
    include_str!("qr.txt")
        .lines()
        .map(|l| l.as_bytes().to_vec())
        .collect()
}

// Write to the screen buffer
fn write_to_buffer(qr: &Vec<Vec<u8>>, output: &mut OutputBuffer) {

    let qr_width = qr[0].len();
    let qr_height = qr.len();
    let square_size: u32 = (min(output.size.x, output.size.y) / max(qr_width as u32, qr_height as u32)) - 2;
    
    let mut offset : Vector2u = Vector2u::new(output.size.x - (square_size * qr_width as u32), output.size.y - (square_size * qr_height as u32));
    offset /= 2;

    let width = qr[0].len() as u32;
    let height = qr.len() as u32;
    for y in 0..height {
        for x in 0..width {
            let mut col: Rgb32 = 0xFFFFFFFF;

            // Black entries, black
            if qr[y as usize][x as usize] == b'B' {
                col = 0x0;
            }

            for yy in 0..square_size {
                for xx in 0..square_size {
                    let index = (y * square_size + yy + offset.y) * output.size.y
                        + (x * square_size + xx + offset.x);
                    output.buffer[index as usize] = col;

                    if (y == 0 && yy == 0) || (y == height - 1 && yy == square_size - 1)
                     || (x == 0 && xx == 0) || (x == width - 1 && xx == square_size - 1) {
                        output.buffer[index as usize] = 0x0;
                    }
                }
            }
        }
    }

}

fn main() {
    let qr = read_qr();

    // Make a 512x512 window
    let window_size = Vector2u::new(512, 512);
    let mut window = Window::new(
        "QR Codes",
        window_size.x as usize,
        window_size.y as usize,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_millis(100)));

    let mut output = OutputBuffer {
        size: window_size,
        buffer: vec![0xFFFFFFFF; (window_size.x * window_size.y) as usize],
    };

    write_to_buffer(&qr, &mut output);

    window
        .update_with_buffer(
            &output.buffer,
            window_size.x as usize,
            window_size.y as usize,
        )
        .unwrap();

    // Window loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(
                &output.buffer,
                window_size.x as usize,
                window_size.y as usize,
            )
            .unwrap();
    }
}

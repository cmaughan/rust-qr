use minifb::{Key, Window, WindowOptions};

use maths_rs::prelude::*;

type Rgb32 = u32;
type Vector2u = Vec2<u32>;
type Vector4u = Vec4<u32>;

// Window output buffer
struct WindowBuffer {
    size: Vector2u,
    buffer: Vec<u32>,
}

fn draw_square(output: &mut WindowBuffer, rc: &Vector4u, col: &Rgb32) {
    for yy in 0..rc.w {
        for xx in 0..rc.z {
            let index = (rc.y * rc.x + yy) * output.size.y + (rc.x * rc.z + xx);
            output.buffer[index as usize] = *col;

            // Outline if desired
            if (rc.y == 0 && yy == 0)
                || (rc.y == rc.w - 1 && yy == rc.w - 1)
                || (rc.x == 0 && xx == 0)
                || (rc.x == rc.z - 1 && xx == rc.z - 1)
            {
                output.buffer[index as usize] = 0x0;
            }
        }
    }
}

// Write to the screen buffer, enlarging the code, putting it in the center, with an outline
fn qr_to_buffer(qr: &QR, output: &mut WindowBuffer) {
    let qr_width = qr.data[0].len() as u32;
    let qr_height = qr.data.len() as u32;
    let square_size: u32 =
        (min(output.size.x, output.size.y) / max(qr_width as u32, qr_height as u32)) - 2;

    let mut offset: Vector2u = Vector2u::new(
        output.size.x - (square_size * qr_width as u32),
        output.size.y - (square_size * qr_height as u32),
    );
    offset /= 2;

    for y in 0..qr_height {
        for x in 0..qr_width {
            let mut col: Rgb32 = 0xFFFFFFFF;

            // Black entries, black
            if qr.data[y as usize][x as usize] == 1 {
                col = 0x0;
            }

            //draw_square(output, &Vector4u::new(x + offset.x, y + offset.y, square_size, square_size), &col);
            for yy in 0..square_size {
                for xx in 0..square_size {
                    let index = (y * square_size + yy + offset.y) * output.size.y
                        + (x * square_size + xx + offset.x);
                    output.buffer[index as usize] = col;

                    // Outline if desired
                    if (y == 0 && yy == 0)
                        || (y == qr_height - 1 && yy == square_size - 1)
                        || (x == 0 && xx == 0)
                        || (x == qr_width - 1 && xx == square_size - 1)
                    {
                        output.buffer[index as usize] = 0x0;
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct QR {
    width: u32,
    height: u32,
    data: Vec<Vec<u8>>,
    version: u32,
    masked: u8,
}

fn read_up(qr: &mut QR, start_coord: &(i32, i32), indices: &Vec<(i32, i32)>) -> u8 {
    let mut number = 0;

    for index in indices {
        let x = (start_coord.0 + index.0) as usize;
        let y = (start_coord.1 + index.1) as usize;
        number = (number << 1) | qr.data[y][x];
    }
    number
}

// Read the QR code
fn read_qr() -> QR {
    let mut data = include_str!("qr.txt")
        .lines()
        .map(|l| {
            l.as_bytes()
                .iter()
                .map(|b| if *b == b'B' { 1 } else { 0 })
                .collect()
        })
        .collect::<Vec<Vec<u8>>>();

    let width = data[0].len() as u32;
    let height = data.len() as u32;

    let version = (data[0].len() as u32 - 17) / 4;
    let mask_bits = data[8][0..5].to_vec();
    dbg!(&mask_bits);
    let mut mask = 0;
    for bit in mask_bits.iter() {
        mask = mask << 1 | bit
    }

    let xor_mask = 0b10101;
    let masked = mask ^ xor_mask;

    for row in 0..height {
        for col in 0..width {
            //let index = (((row * col) % 2) + ((row * col) % 3)) % 2;
            let index = (((row * col) % 3) + (row * col)) % 2;
            if index == 0 {
                if data[row as usize][col as usize] == 0 {
                    data[row as usize][col as usize] = 1;
                } else {
                    data[row as usize][col as usize] = 0;
                }
            }
        }
    }

    let mut q = QR {
        width,
        height,
        data,
        version,
        masked,
    };

    let up_indices_4: Vec<(i32, i32)> = vec![(1, 1), (0, 1), (1, 0), (0, 0)];

    let up_indices_8: Vec<(i32, i32)> = vec![
        (1, 3),
        (0, 3),
        (1, 2),
        (0, 2),
        (1, 1),
        (0, 1),
        (1, 0),
        (0, 0),
    ];

    let down_indices_8: Vec<(i32, i32)> = vec![
        (1, 0),
        (0, 0),
        (1, 1),
        (0, 1),
        (1, 2),
        (0, 2),
        (1, 3),
        (0, 3),
    ];

    let left_indices_8: Vec<(i32, i32)> = vec![
        (3, 1),
        (2, 1),
        (3, 0),
        (2, 0),
        (1, 1),
        (0, 1),
        (1, 0),
        (0, 0),
    ];
    
    let left_up_indices_8: Vec<(i32, i32)> = vec![
        (3, 0),
        (2, 0),
        (3, 1),
        (2, 1),
        (1, 1),
        (0, 1),
        (1, 0),
        (0, 0),
    ];
    
    let winding: Vec<(i32, i32, &Vec<(i32, i32)>)> = vec![
        (0, 0, &up_indices_8),
        (-2, -2, &left_indices_8),
        (0, 2, &down_indices_8),
        (0, 4, &down_indices_8),
        (-2, 4, &left_up_indices_8),
        (0, -4, &up_indices_8),
        (0, -4, &up_indices_8),
        (-2, -2, &left_indices_8),
        (0, 2, &down_indices_8),
        (0, 4, &down_indices_8),
        (-2, 4, &left_up_indices_8),
        (0, -4, &up_indices_8),
    ];

    let mut current_x = (width - 2) as i32;
    let mut current_y = (height - 2) as i32;

    let encoding = read_up(&mut q, &(current_x, current_y), &up_indices_4);
    current_y -= 4;
    let length = read_up(&mut q, &(current_x, current_y), &up_indices_8);
    current_y -= 4;

    let mut name : String = "".to_string();
    for wind in winding {
        current_y += wind.1;
        current_x += wind.0;
        name.push(read_up(&mut q, &(current_x, current_y), wind.2) as char);
    }

    dbg!(encoding);
    dbg!(encoding);
    dbg!(length);
    dbg!(name);
    q
}

fn main() {
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

    let mut output = WindowBuffer {
        size: window_size,
        buffer: vec![0xFFFFFFFF; (window_size.x * window_size.y) as usize],
    };

    let qr = read_qr();

    dbg!(qr.width);
    dbg!(qr.height);
    dbg!(qr.version);
    dbg!(&qr.masked);

    qr_to_buffer(&qr, &mut output);

    // Update the window contents
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

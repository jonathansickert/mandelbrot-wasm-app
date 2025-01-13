use image::{ImageBuffer, RgbImage};
use num_complex::Complex;
use rayon::prelude::*;
use std::env;
use std::time::Instant;

const MAX_ITER: usize = 255;
const WIDTH: usize = 2560; //2560; 16384
const HEIGHT: usize = 1600; //1600; 16384

const MANDELBROT_X_START: f64 = -2.0;
const MANDELBROT_X_END: f64 = 2.0;
const MANDELBROT_Y_START: f64 = -2.0;
const MANDELBROT_Y_END: f64 = 2.0;

fn _mandelbrot_naive_smoothed(x: f64, y: f64) -> f64 {
    let c: Complex<f64> = Complex::new(x, y);
    let mut z: Complex<f64> = c;
    let mut iter: f64 = 0.0;

    while z.norm_sqr() <= 4.0 && iter < MAX_ITER as f64 {
        z = z.powu(2) + c;
        iter += 1.0;
    }

    if iter < MAX_ITER as f64 {
        let log_zn: f64 = z.norm().ln();
        let nu: f64 = log_zn.ln() / std::f64::consts::LN_2;
        iter = iter + 1.0 - nu;
    }

    return iter;
}

fn iters_to_colors_smooved(iter: f64, gradient: colorous::Gradient) -> [u8; 3] {
    if iter == MAX_ITER as f64 {
        return [0, 0, 0];
    }
    let color: colorous::Color = gradient.eval_continuous(iter / MAX_ITER as f64);
    return [color.r, color.g, color.b];
}

fn mandelbrot_naive_smoothed(
    x_start: f64,
    x_end: f64,
    y_start: f64,
    y_end: f64,
    vec: &mut [u8],
    gradient: colorous::Gradient,
) {
    let dx: f64 = (x_end - x_start) / (WIDTH - 1) as f64;
    let dy: f64 = (y_end - y_start) / (HEIGHT - 1) as f64;

    vec.par_chunks_exact_mut(WIDTH * 3)
        .enumerate()
        .for_each(|(row, chunk)| {
            let y: f64 = y_start + dy * row as f64;
            for col in 0..WIDTH {
                let x: f64 = x_start + dx * col as f64;
                let iter: f64 = _mandelbrot_naive_smoothed(x, y);
                let rgb: [u8; 3] = iters_to_colors_smooved(iter, gradient);
                chunk[col * 3 + 0] = rgb[0];
                chunk[col * 3 + 1] = rgb[1];
                chunk[col * 3 + 2] = rgb[2];
            }
        });
}

fn mandelbrot_naive_scaled(
    x_center: f64,
    y_center: f64,
    zoom: f64,
    vec: &mut [u8],
    gradient: colorous::Gradient,
) {
    let mut x_start: f64 = x_center + MANDELBROT_X_START / zoom;
    let mut x_end: f64 = x_center + MANDELBROT_X_END / zoom;
    let mut y_start: f64 = y_center + MANDELBROT_Y_START / zoom;
    let mut y_end: f64 = y_center + MANDELBROT_Y_END / zoom;

    let x_range: f64 = x_end - x_start;
    let y_range: f64 = y_end - y_start;
    let image_aspect_ratio: f64 = WIDTH as f64 / HEIGHT as f64;
    let mandelbrot_aspect_ratio: f64 = x_range / y_range;
    let adjustment_factor: f64 = image_aspect_ratio / mandelbrot_aspect_ratio;

    if adjustment_factor > 1.0 {
        let diff: f64 = (x_range * adjustment_factor - x_range) / 2.0;
        x_start -= diff;
        x_end += diff;
    } else {
        let diff: f64 = (y_range / adjustment_factor - y_range) / 2.0;
        y_start -= diff;
        y_end += diff;
    }

    print!("{}, {}, {}, {}\n", x_start, x_end, y_start, y_end);

    mandelbrot_naive_smoothed(x_start, x_end, y_start, y_end, vec, gradient);
}

fn main() {
    let start_mandelbrot: Instant = std::time::Instant::now();
    let mut vec: Box<[u8]> = vec![0; WIDTH * 3 * HEIGHT].into_boxed_slice();

    let args: Vec<String> = env::args().collect();

    let x_center: f64 = match args[1].parse() {
        Ok(num) => num,
        Err(_) => {
            eprintln!("Error: '{}' is not a valid float64.", args[1]);
            std::process::exit(1);
        }
    };

    let y_center: f64 = match args[2].parse() {
        Ok(num) => num,
        Err(_) => {
            eprintln!("Error: '{}' is not a valid float64.", args[2]);
            std::process::exit(1);
        }
    };

    let zoom: f64 = match args[3].parse() {
        Ok(num) => num,
        Err(_) => {
            eprintln!("Error: '{}' is not a valid integer.", args[3]);
            std::process::exit(1);
        }
    };

    mandelbrot_naive_scaled(x_center, y_center, zoom, &mut vec, colorous::TURBO);

    print!("Mandelbrot: {}\n", start_mandelbrot.elapsed().as_secs_f32());

    let start_saving: Instant = std::time::Instant::now();
    let img: RgbImage = ImageBuffer::from_raw(WIDTH as u32, HEIGHT as u32, vec.to_vec())
        .expect("Failed to create image buffer");

    img.save("output.png").unwrap();

    print!("Saving: {}\n", start_saving.elapsed().as_secs_f32());
}

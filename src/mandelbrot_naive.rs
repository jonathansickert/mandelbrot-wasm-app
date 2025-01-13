use num_complex::Complex;
use rayon::prelude::*;

const MAX_ITER: usize = 200;
const WIDTH: usize = 2560;
const HEIGHT: usize = 1600;

const MANDELBROT_X_START: f64 = -2.0;
const MANDELBROT_X_END: f64 = 0.47;
const MANDELBROT_Y_START: f64 = -1.12;
const MANDELBROT_Y_END: f64 = 1.12;

fn _mandelbrot_naive(x: f64, y: f64) -> usize {
    let c: Complex<f64> = Complex::new(x, y);
    let mut z: Complex<f64> = c;
    let mut iter: usize = 0;

    while z.re * z.re + z.im + z.im <= 4.0 && iter < MAX_ITER {
        z = z.powu(2) + c;
        iter += 1;
    }

    return iter;
}

fn mandelbrot_naive(x_start: f64, x_end: f64, y_start: f64, y_end: f64, vec: &mut [usize]) {
    let dx: f64 = (x_end - x_start) / (WIDTH - 1) as f64;
    let dy: f64 = (y_end - y_start) / (HEIGHT - 1) as f64;

    vec.par_chunks_mut(WIDTH)
        .enumerate()
        .for_each(|(row, chunk)| {
            for col in 0..WIDTH {
                let x: f64 = x_start + dx * col as f64;
                let y: f64 = y_start + dy * row as f64;
                chunk[col] = _mandelbrot_naive(x, y);
            }
        });
}

fn mandelbrot_naive_scaled(x_center: f64, y_center: f64, zoom: f64, vec: &mut [usize]) {
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

    mandelbrot_naive(x_start, x_end, y_start, y_end, vec);
}

fn iters_to_colors(iter: usize, gradient: colorous::Gradient) -> [u8; 3] {
    if iter == MAX_ITER {
        return [0, 0, 0];
    }
    let color: colorous::Color = gradient.eval_rational(iter, MAX_ITER);
    return [color.r, color.g, color.b];
}

// fn iters_to_rbg(iter: usize, colormap: Vec<(f32, f32, f32)>) -> [u8; 3] {
//     if iter == MAX_ITER {
//         return [0, 0, 0];
//     }

//     let normalized_iter: f32 = (iter as f32) / MAX_ITER as f32;

//     let i: usize = (normalized_iter * (colormap.len() as f32 - 1.0)) as usize;
//     let j: usize = (i + 1).min(colormap.len() - 1);

//     let t: f32 = normalized_iter * (colormap.len() as f32 - 1.0) - i as f32;
//     let (r1, g1, b1) = colormap[i];
//     let (r2, g2, b2) = colormap[j];

//     let r: f32 = (r1 + t * (r2 - r1)) * MAX_ITER as f32;
//     let g: f32 = (g1 + t * (g2 - g1)) * MAX_ITER as f32;
//     let b: f32 = (b1 + t * (b2 - b1)) * MAX_ITER as f32;

//     return [r as u8, g as u8, b as u8];
// }

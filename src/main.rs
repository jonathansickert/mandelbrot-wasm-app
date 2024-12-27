use image::{ImageBuffer, RgbImage};
use palette::{IntoColor, Lch, Srgb};

const MAX_ITER: u8 = std::u8::MAX;

fn mandelbrot(x: f64, y: f64) -> u8 {
    let mut re: f64 = x;
    let mut im: f64 = y;

    for i in 0..MAX_ITER {
        let re2: f64 = re * re;
        let im2: f64 = im * im;

        if re2 + im2 > 4.0 {
            return i;
        }

        im = 2.0 * re * im + y;
        re = re2 - im2 + x;
    }

    return MAX_ITER;
}

fn naive_mandelbrot(
    width: usize,
    height: usize,
    x_start: f64,
    x_end: f64,
    y_start: f64,
    y_end: f64,
    vec: &mut Vec<u8>,
) {
    let dx: f64 = (x_end - x_start) / (width - 1) as f64;
    let dy: f64 = (y_end - y_start) / (height - 1) as f64;

    for row in 0..height {
        for col in 0..width {
            let x: f64 = x_start + dx * col as f64;
            let y: f64 = y_end - dy * row as f64;
            vec[row * width + col] = mandelbrot(x, y);
        }
    }
}

// fn intensity_to_rainbow(v: u8) -> Rgb<u8> {
//     let colors = [
//         Rgb([148, 0, 211]), // Violet
//         Rgb([75, 0, 130]),  // Indigo
//         Rgb([0, 0, 255]),   // Blue
//         Rgb([0, 255, 0]),   // Green
//         Rgb([255, 255, 0]), // Yellow
//         Rgb([255, 127, 0]), // Orange
//         Rgb([255, 0, 0]),   // Red
//     ];

//     let index = (v as usize * (colors.len() - 1)) / 255;
//     colors[index]
// }

fn iters_to_rbg(iter: u8) -> Vec<u8> {
    let colors: [(f32, f32, f32); 5] = [
        (0.0, 0.0, 0.0), // Black
        (1.0, 0.0, 0.0), // Red
        (0.0, 0.0, 1.0), // Blue
        (0.0, 1.0, 0.0), // Green
        (0.0, 0.0, 0.0), // Black
    ];

    let normalized_iter: f32 = (iter as f32) / 255.0;

    // Compute which two colors to interpolate between
    let index = (norm_value * (colors.len() as f32 - 1.0)) as usize;
    let next_index = (index + 1).min(colors.len() - 1);

    // Interpolate between the two colors
    let t = norm_value * (colors.len() as f32 - 1.0) - index as f32;
    let (r1, g1, b1) = colors[index];
    let (r2, g2, b2) = colors[next_index];

    // Linear interpolation between colors
    let r = r1 + t * (r2 - r1);
    let g = g1 + t * (g2 - g1);
    let b = b1 + t * (b2 - b1);

    return vec![(r * 255.0) as u8, (b * 255.0) as u8, (g * 255.0) as u8];
}

fn intensity_to_hsv(iter_count: u8) -> Vec<f32> {
    let normalized_iter = iter_count as f32 / 255.0;
    return vec![
        (normalized_iter * 360.0).powf(1.5) % 360.0,
        100.0,
        normalized_iter * 100.0,
    ];
}

fn lch_to_rgb(lch: Vec<f32>) -> Vec<u8> {
    let lch: Lch = Lch::new(lch[0], lch[1], lch[2]);
    let rgb: Srgb = Srgb::from_linear(lch.into_color());

    let r = (rgb.red * 255.0).clamp(0.0, 255.0).round() as u8;
    let g = (rgb.green * 255.0).clamp(0.0, 255.0).round() as u8;
    let b = (rgb.blue * 255.0).clamp(0.0, 255.0).round() as u8;

    return vec![r, b, g];
}

fn intensity_to_lch(iters: u8) -> Vec<f32> {
    let s: f32 = iters as f32 / 255.0;
    let v: f32 = 1.0 - (std::f32::consts::PI * s).cos().powf(2.0);
    return vec![
        75.0 - (75.0 * v),
        28.0 + (75.0 - (75.0 * v)),
        (s * 360.0).powf(1.5) % 360.0,
    ];
}

// fn iter_count_to_rgb(iter_count: u8) -> Vec<u8> {
//     let s: f32 = 100.0;
//     let n: f32 = 100.0;
//     let v: f32 = ((iter_count as f32 / 255.0).powf(s) * n).powf(1.5) % n;
//     let c: u8 = (v * 255.0) as u8;
//     return vec![c, c, c];
// }

fn hsv_to_rgb(hsv: Vec<f32>) -> Vec<u8> {
    let h = hsv[0].clamp(0.0, 360.0); // Hue: 0-360 degrees
    let s = hsv[1].clamp(0.0, 1.0); // Saturation: 0-1
    let v = hsv[2].clamp(0.0, 1.0); // Value: 0-1

    let c = v * s; // Chroma
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r_prime, g_prime, b_prime) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let r: u8 = ((r_prime + m) * 255.0).round() as u8;
    let g: u8 = ((g_prime + m) * 255.0).round() as u8;
    let b: u8 = ((b_prime + m) * 255.0).round() as u8;

    return vec![r, g, b];
}

fn main() {
    let width: usize = 3000;
    let height: usize = 3000;
    let mut vec: Vec<u8> = vec![0; width * height];

    naive_mandelbrot(width, height, -1.0, 0.0, 0.0, 1.0, &mut vec);

    print!("done!\n");

    let img: RgbImage = ImageBuffer::from_raw(
        width as u32,
        height as u32,
        vec.into_iter()
            .flat_map(|v| iters_to_rbg(v))
            .collect::<Vec<u8>>(),
    )
    .expect("Failed to create image buffer");

    // Save the image
    img.save("img3.png").unwrap();
}

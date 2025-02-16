use colorous::{Color, Gradient};
use num_complex::Complex;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, ImageData, WheelEvent};

const MAX_ITER: usize = 255;

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

fn iters_to_colors_smooved(iter: f64, gradient: Gradient) -> [u8; 3] {
    if iter == MAX_ITER as f64 {
        return [0, 0, 0];
    }
    let color: Color = gradient.eval_continuous(iter / MAX_ITER as f64);
    return [color.r, color.g, color.b];
}

fn mandelbrot_naive_smoothed(
    x_start: f64,
    x_end: f64,
    y_start: f64,
    y_end: f64,
    width: f64,
    height: f64,
    vec: &mut Vec<u8>,
    gradient: Gradient,
) {
    let dx: f64 = (x_end - x_start) / (width - 1.0);
    let dy: f64 = (y_end - y_start) / (height - 1.0);

    for row in 0..height as usize {
        for col in 0..width as usize {
            let y: f64 = y_start + dy * row as f64;
            let x: f64 = x_start + dx * col as f64;
            let iter: f64 = _mandelbrot_naive_smoothed(x, y);
            let rgb: [u8; 3] = iters_to_colors_smooved(iter, gradient);
            let idx: usize = (row * width as usize + col) * 4;
            vec[idx + 0] = rgb[0];
            vec[idx + 1] = rgb[1];
            vec[idx + 2] = rgb[2];
            vec[idx + 3] = 255;
        }
    }
}

fn scale_bounds(
    bounds: &Bounds,
    width: f64,
    height: f64,
    mouse_x: f64,
    mouse_y: f64,
    zoom: f64,
) -> Bounds {
    let x_range = bounds.x_end - bounds.x_start;
    let y_range = bounds.y_end - bounds.y_start;
    let canvas_aspect_ratio = width / height;

    let mouse_rel_x = bounds.x_start + (mouse_x / width) * x_range;
    let mouse_rel_y = bounds.y_start + (mouse_y / height) * y_range;

    let mut x_start = mouse_rel_x - (mouse_x / width) * (x_range / zoom);
    let mut x_end = mouse_rel_x + ((width - mouse_x) / width) * (x_range / zoom);
    let mut y_start = mouse_rel_y - (mouse_y / height) * (y_range / zoom);
    let mut y_end = mouse_rel_y + ((height - mouse_y) / height) * (y_range / zoom);

    if canvas_aspect_ratio > 1.0 {
        let adjusted_x_range = (y_end - y_start) * canvas_aspect_ratio;
        let x_center = (x_start + x_end) / 2.0;
        x_start = x_center - adjusted_x_range / 2.0;
        x_end = x_center + adjusted_x_range / 2.0;
    } else {
        let adjusted_y_range = (x_end - x_start) / canvas_aspect_ratio;
        let y_center = (y_start + y_end) / 2.0;
        y_start = y_center - adjusted_y_range / 2.0;
        y_end = y_center + adjusted_y_range / 2.0;
    }

    Bounds {
        x_start,
        x_end,
        y_start,
        y_end,
    }
}

fn draw_mandelbrot(
    x_start: f64,
    x_end: f64,
    y_start: f64,
    y_end: f64,
    width: f64,
    height: f64,
    gradient: Gradient,
    canvas: &HtmlCanvasElement,
) {
    let mut vec: Vec<u8> = vec![0; 4 * width as usize * height as usize];

    mandelbrot_naive_smoothed(
        x_start, x_end, y_start, y_end, width, height, &mut vec, gradient,
    );

    let js_array = js_sys::Uint8ClampedArray::new_with_length(vec.len() as u32);
    js_array.copy_from(&vec);

    let data: ImageData =
        ImageData::new_with_js_u8_clamped_array_and_sh(&js_array, width as u32, height as u32)
            .expect("");

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    ctx.put_image_data(&data, 0.0, 0.0)
        .expect("error while drawing image");
}

struct Bounds {
    x_start: f64,
    x_end: f64,
    y_start: f64,
    y_end: f64,
}

impl Bounds {
    fn new() -> Self {
        Self {
            x_start: -2.0,
            x_end: 2.0,
            y_start: -2.0,
            y_end: 2.0,
        }
    }
}

#[wasm_bindgen]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = window().expect("no window");
    let document = window.document().expect("no document");
    let canvas = document
        .get_element_by_id("fullscreenCanvas")
        .expect("no canvas")
        .dyn_into::<HtmlCanvasElement>()
        .expect("failed to cast");

    let width = canvas.client_width() as f64;
    let height = canvas.client_height() as f64;
    canvas.set_width(width as u32);
    canvas.set_height(height as u32);

    let bounds = Rc::new(RefCell::new(scale_bounds(
        &Bounds::new(),
        width,
        height,
        0.0,
        0.0,
        1.0,
    )));

    draw_mandelbrot(
        bounds.borrow().x_start,
        bounds.borrow().x_end,
        bounds.borrow().y_start,
        bounds.borrow().y_end,
        width,
        height,
        colorous::TURBO,
        &canvas,
    );

    let timeout_handle = Rc::new(RefCell::new(None::<i32>));
    let transition_duration = 200;

    let smooth_transform_closure = {
        let bounds = bounds.clone();
        let timeout_handle = timeout_handle.clone();
        let window = window.clone();
        let canvas = canvas.clone();

        Closure::<dyn FnMut(WheelEvent)>::new(move |event: WheelEvent| {
            if let Some(handle) = *timeout_handle.borrow() {
                window.clear_timeout_with_handle(handle);
            }

            let zoom: f64 = if event.delta_y() < 0.0 { 2.0 } else { 0.5 };
            let mouse_x = event.offset_x() as f64;
            let mouse_y = event.offset_y() as f64;
            let transform_origin_x = (mouse_x / width) * 100.0;
            let transform_origin_y = (mouse_y / height) * 100.0;

            canvas
                .style()
                .set_property("transform", &format!("scale({})", zoom))
                .unwrap();
            canvas
                .style()
                .set_property(
                    "transform-origin",
                    &format!("{}% {}%", transform_origin_x, transform_origin_y),
                )
                .unwrap();
            canvas
                .style()
                .set_property(
                    "transition",
                    &format!("transform {}ms ease", transition_duration),
                )
                .unwrap();

            let draw_callback = {
                let bounds = bounds.clone();
                let timeout_handle = timeout_handle.clone();
                let canvas = canvas.clone();

                Closure::<dyn FnOnce()>::once(move || {
                    canvas.style().remove_property("transition").unwrap();
                    canvas.style().remove_property("transform").unwrap();

                    let new_bounds =
                        scale_bounds(&*bounds.borrow(), width, height, mouse_x, mouse_y, zoom);
                    bounds.borrow_mut().x_start = new_bounds.x_start;
                    bounds.borrow_mut().x_end = new_bounds.x_end;
                    bounds.borrow_mut().y_start = new_bounds.y_start;
                    bounds.borrow_mut().y_end = new_bounds.y_end;

                    draw_mandelbrot(
                        bounds.borrow().x_start,
                        bounds.borrow().x_end,
                        bounds.borrow().y_start,
                        bounds.borrow().y_end,
                        width,
                        height,
                        colorous::TURBO,
                        &canvas,
                    );
                    *timeout_handle.borrow_mut() = None;
                })
            };

            let id = window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    draw_callback.as_ref().unchecked_ref(),
                    transition_duration,
                )
                .unwrap();

            *timeout_handle.borrow_mut() = Some(id);
            draw_callback.forget();
        })
    };

    canvas
        .add_event_listener_with_callback(
            "wheel",
            smooth_transform_closure.as_ref().unchecked_ref(),
        )
        .unwrap();

    smooth_transform_closure.forget();
    Ok(())
}

use image::{ImageBuffer, Rgb};
use num_complex::Complex;
use std::time::Instant;

fn main() {
    let start = Instant::now(); // ★ 計測開始
    let bounds = (1200, 800);
    let upper_left = Complex::new(-2.2, 1.2);
    let lower_right = Complex::new(1.0, -1.2);
    let max_iter = 200;

    let mut pixels = Vec::with_capacity(bounds.0 * bounds.1 * 3);

    for y in 0..bounds.1 {
        for x in 0..bounds.0 {
            let point = pixel_to_point(bounds, (x, y), upper_left, lower_right);
            let iter = escape_time(point, max_iter);
            let [r, g, b] = color_map(iter, max_iter);
            pixels.extend_from_slice(&[r, g, b]);
        }
    }

    write_image("mandelbrot.png", &pixels, bounds).unwrap();
    let elapsed = start.elapsed(); // ★ 経過時間 
    println!( "mandelbrot.png を生成しました！\n処理時間: {:.3} 秒", elapsed.as_secs_f64() );
}

/// ピクセル座標 → 複素平面上の点
fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Complex<f64> {
    let (width, height) = (lower_right.re - upper_left.re, upper_left.im - lower_right.im);

    Complex::new(
        upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        upper_left.im - pixel.1 as f64 * height / bounds.1 as f64,
    )
}

/// マンデルブロ集合の発散判定
fn escape_time(c: Complex<f64>, max_iter: u32) -> u32 {
    let mut z = Complex::new(0.0, 0.0);

    for i in 0..max_iter {
        if z.norm_sqr() > 4.0 {
            return i;
        }
        z = z * z + c;
    }
    max_iter
}

/// 反復回数 → RGB 色変換（滑らかなグラデーション）
fn color_map(iter: u32, max_iter: u32) -> [u8; 3] {
    if iter >= max_iter {
        return [0, 0, 0]; // 内部は緑
    }

    let t = iter as f32 / max_iter as f32;

    // 有名な smooth coloring（青→紫→赤→黄）
    let r = (9.0 * (1.0 - t) * t * t * t * 255.0) as u8;
    let g = (15.0 * (1.0 - t) * (1.0 - t) * t * t * 255.0) as u8;
    let b = (8.5 * (1.0 - t) * (1.0 - t) * (1.0 - t) * t * 255.0) as u8;

    [r, g, b]
}

/// 画像保存
fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize)) -> image::ImageResult<()> {
    let buffer: ImageBuffer<Rgb<u8>, _> =
        ImageBuffer::from_raw(bounds.0 as u32, bounds.1 as u32, pixels.to_vec())
            .expect("buffer size mismatch");

    buffer.save(filename)
}


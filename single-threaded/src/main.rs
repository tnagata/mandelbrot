#![allow(unused)]

use num::Complex;
use std::time::Instant;
/*
c がマンデルブロ集合に属するかどうかを、最大 limit 回の反復で判定する。

c が集合の要素でない場合は Some(i) を返す。
ここで i は、原点を中心とする半径 2 の円から c が外に出るまでに必要だった反復回数である。

もし c が集合の要素であるように見える場合（より正確には、c が集合に属さないと証明できないまま
反復回数の上限に達した場合）は、None を返す。
*/
fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }

    None
}

use std::str::FromStr;

/*
`s` を座標のペアとしてパースする。例えば `"400x600"` や `"1.0,0.5"` のような文字列である。

より具体的には、`s` は `<left><sep><right>` という形式をしていなければならない。
ここで `<sep>` は `separator` 引数で与えられた 1 文字の区切り文字であり、`<left>` と `<right>` は
どちらも `T::from_str` でパース可能な文字列である。

`s` がこの形式に従っていれば、`Some<(x, y)>` を返す。正しくパースできなかった場合は `None` を返す。
*/

fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None
            }
        }
    }
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("",        ','), None);
    assert_eq!(parse_pair::<i32>("10,",     ','), None);
    assert_eq!(parse_pair::<i32>(",10",     ','), None);
    assert_eq!(parse_pair::<i32>("10,20",   ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
    assert_eq!(parse_pair::<f64>("0.5x",    'x'), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}

// カンマで区切られた 2 つの浮動小数点数をパースして、複素数として解釈する。
fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None
    }
}

#[test]
fn test_parse_complex() {
    assert_eq!(parse_complex("1.25,-0.0625"),
               Some(Complex { re: 1.25, im: -0.0625 }));
    assert_eq!(parse_complex(",-0.0625"), None);
}

/*
出力画像のあるピクセルの行と列から、複素平面上の対応する点を返す。

bounds は画像の幅と高さ（ピクセル数）を表すペア。
pixel は画像内の特定のピクセルを示す (列, 行) のペア。
upper_left と lower_right は、画像がカバーする複素平面上の領域を示す 2 点である。
*/

fn pixel_to_point(bounds: (usize, usize),
                  pixel: (usize, usize),
                  upper_left: Complex<f64>,
                  lower_right: Complex<f64>)
    -> Complex<f64>
{
    let (width, height) = (lower_right.re - upper_left.re,
                           upper_left.im - lower_right.im);
    Complex {
        re: upper_left.re + pixel.0 as f64 * width  / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64
        // Why subtraction here? pixel.1 increases as we go down,
        // but the imaginary component increases as we go up.
    }
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(pixel_to_point((100, 200), (25, 175),
                              Complex { re: -1.0, im:  1.0 },
                              Complex { re:  1.0, im: -1.0 }),
               Complex { re: -0.5, im: -0.75 });
}

/*
マンデルブロ集合のある矩形領域を、ピクセルバッファへ描画する。

bounds 引数は、1 バイトにつき 1 つのグレースケール値を持つピクセルバッファで pixels の幅と高さを表す。
upper_left と lower_right は、ピクセルバッファの左上および右下の角に対応する複素平面上の点を指定する。
*/
fn render(pixels: &mut [u8],
          bounds: (usize, usize),
          upper_left: Complex<f64>,
          lower_right: Complex<f64>)
{
    assert!(pixels.len() == bounds.0 * bounds.1);

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column, row),
                                       upper_left, lower_right);
            pixels[row * bounds.0 + column] =
                match escape_time(point, 255) {
                    None => 0,
                    Some(count) => 255 - count as u8
                };
        }
    }
}


/// 全面変更
use image::{ImageBuffer, Luma, ImageError};

fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize))
    -> Result<(), ImageError>
{
    let buffer: ImageBuffer<Luma<u8>, _> =
        ImageBuffer::from_raw(bounds.0 as u32, bounds.1 as u32, pixels.to_vec())
            .expect("buffer size mismatch");

    buffer.save(filename)?;
    Ok(())
}

fn main() {
    /*
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        eprintln!("Usage: {} FILE PIXELS UPPERLEFT LOWERRIGHT",
                  args[0]);
        eprintln!("Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
                  args[0]);
        std::process::exit(1);
    }

    let bounds = parse_pair(&args[2], 'x')
        .expect("error parsing image dimensions");
    let upper_left = parse_complex(&args[3])
        .expect("error parsing upper left corner point");
    let lower_right = parse_complex(&args[4])
        .expect("error parsing lower right corner point");
*/
    let start = Instant::now(); // ★ 計測開始
    let bounds = (1200, 800);
    let upper_left = Complex::new(-2.2, 1.2);
    let lower_right = Complex::new(1.0, -1.2);
    let mut pixels = vec![0; bounds.0 * bounds.1];

    render(&mut pixels, bounds, upper_left, lower_right);
//    write_image(&args[1], &pixels, bounds)
    write_image("mandelbrot.png", &pixels, bounds)
        .expect("error writing PNG file");
    let elapsed = start.elapsed(); // ★ 経過時間 
    println!( "mandelbrot.png を生成しました！\n処理時間: {:.3} 秒", elapsed.as_secs_f64() );
}


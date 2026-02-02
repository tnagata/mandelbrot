mandelbrotの説明　(single-threaded/main.rsを参照）：
この `main.rs` ファイルは、マンデルブロ集合という美しいフラクタル図形を生成し、PNG画像として出力するプログラムです。Rustというプログラミング言語で書かれています。

コードをいくつかの部分に分けて、それぞれ何をしているのか説明しますね。

### 1. `escape_time` 関数
この関数は、ある複素数 `c` がマンデルブロ集合に属するかどうかを調べます。マンデルブロ集合の定義に基づいて、`z = z*z + c` という計算を繰り返し行い、`z` の大きさが2を超えたらマンデルブロ集合には属さないと判断します。`limit` は計算を繰り返す最大回数です。

*   もし `z` の大きさが2を超えたら、その時の繰り返し回数 `i` を返します。
*   `limit` 回繰り返しても大きさが2を超えなかったら、マンデルブロ集合に属するとみなして `None` を返します。

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


### 2. `parse_pair` 関数
この関数は、例えば `"400x600"` のような文字列を `"400"` と `"600"` に分け、それぞれを数値として解析するものです。区切り文字（`x` や `,` など）を指定できます。成功すれば `(数値, 数値)` のタプルを、失敗すれば `None` を返します。

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


### 3. `parse_complex` 関数
これは `parse_pair` 関数を使って、`"1.25,-0.0625"` のような文字列を複素数（`Complex { re: 1.25, im: -0.0625 }`）に変換します。

fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, \',\') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None
    }
}


### 4. `pixel_to_point` 関数
これは、出力画像の特定のピクセル（行と列）に対応する、複素平面上の点を返します。
 `bounds` は画像の幅と高さ（ピクセル数）を表すペアです。
`pixel` は画像内の特定のピクセルを示す（列, 行）のペアです。
`upper_left` と `lower_right` は、画像がカバーする複素平面上の領域を示す点です。

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
        //なぜここで減算するのか？
// pixel.1（行番号）は下方向へ進むほど値が大きくなるが、
//複素数の虚部は上方向へ進むほど値が大きくなるため。
//画像座標系と複素平面のy軸の向きが逆なので、値を反転させる必要があるため
    }
}

### 5. `render` 関数
この関数がマンデルブロ集合の画像を実際に生成する部分です。ピクセルごとに `pixel_to_point` で複素平面上の点を計算し、`escape_time` で
その点がマンデルブロ集合に属するかどうかを調べます。結果に応じて、ピクセルに色の値（0から255のグレースケール値）を設定します。
マンデルブロ集合に属すると判断された点（`None`）は黒（0）になり、属さない点はその繰り返し回数に基づいて明るい色になります。

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


### 6. `write_image` 関数
`render` 関数で生成されたピクセルのデータを、指定されたファイル名でPNG画像として保存します。
　　最新のimageクレートでコンパイルエラーが出るので全面変更した
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


### 7. `main` 関数
これはプログラムのエントリポイント（一番最初に実行される部分）です。
元のコメントアウトされているソースコードの説明。
*   コマンドライン引数（プログラム実行時に与える情報）を解析します。
    *   引数が5つ必要で、そうでなければ使い方を表示して終了します。
    *   `FILE`: 出力するPNGファイルの名前
    *   `PIXELS`: 画像の幅と高さ（例: `1000x750`）
    *   `UPPERLEFT`: マンデルブロ集合を描画する領域の左上隅の複素数（例: `-1.20,0.35`）
    *   `LOWERRIGHT`: マンデルブロ集合を描画する領域の右下隅の複素数（例: `-1,0.20`）
*   `parse_pair` と `parse_complex` を使って、引数の文字列を数値や複素数に変換します。
*   `pixels` というバイト配列（画像データが格納される場所）を準備します。
fn main() {
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

    let mut pixels = vec![0; bounds.0 * bounds.1];

    // Scope of slicing up `pixels` into horizontal bands.
    {
        let bands: Vec<(usize, &mut [u8])> = pixels
            .chunks_mut(bounds.0)
            .enumerate()
            .collect();

        bands.into_par_iter()
            .for_each(|(i, band)| {
                let top = i;
                let band_bounds = (bounds.0, 1);
                let band_upper_left = pixel_to_point(bounds, (0, top),
                                                     upper_left, lower_right);
                let band_lower_right = pixel_to_point(bounds, (bounds.0, top + 1),
                                                      upper_left, lower_right);
                render(band, band_bounds, band_upper_left, band_lower_right);
            });
    }

    write_image(&args[1], &pixels, bounds)
        .expect("error writing PNG file");
}

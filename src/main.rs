use std::str::FromStr;

use crossbeam::thread::Scope;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use num_complex::Complex64;
use std::fs::File;
use std::io::Write;

macro_rules! cmplx {
    () => {
        Complex64::new(0.0, 0.0)
    };

    ($re: expr) => {
        Complex64::new($re, $re)
    };

    ($re: expr, $im: expr) => {
        Complex64::new($re, $im)
    };
}

fn main() {
    const THREADS: u8 = 8;
    const MSG1: &'static str = "USAGE: mandelbrot <file> <pixels> <upper_left> <lower_right>";
    const MSG2: &'static str = "mandel.png 4000x3000 -1.20,0.35 -1,0.20";

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        let pname: &str = &args[0][..];
        writeln!(std::io::stderr(), "{}\nEXAMPLE: {} {}", MSG1, pname, MSG2).unwrap();
        std::process::exit(1);
    }

    let bounds = parse_pair(&args[2], 'x').expect("Error parsing image dimensions");
    let u_l = parse_complex(&args[3]).expect("Error parsing upper left corner point");
    let l_r = parse_complex(&args[4]).expect("Error parsing lower right corner point");
    let mut pixels = vec![0; bounds.0 * bounds.1];

    render(&mut pixels, bounds, u_l, l_r);
    write_image(&args[1], &pixels, bounds).expect("Error writing PNG file");

    let rows_per_band = bounds.1 / usize::from(THREADS + 1 as u8);

    {
        let bands: Vec<&mut [u8]> = pixels.chunks_mut(rows_per_band * bounds.0).collect();

        crossbeam::scope(|spawner| {
            for (i, band) in bands.into_iter().enumerate() {
                let top = rows_per_band * i;
                let height = band.len() / bounds.0;
                let band_bounds = (bounds.0, height);
                let band_upper_left = pixel_to_point(bounds, (0, top), u_l, l_r);
                let band_lower_right = pixel_to_point(bounds, (bounds.0, top + height), u_l, l_r);
                spawner.spawn(move |_: &Scope| {
                    render(band, band_bounds, band_upper_left, band_lower_right);
                });
            }
        })
        .unwrap();
    }
}

fn escape_time(c: Complex64, limit: u32) -> Option<u32> {
    let mut z = cmplx!();
    for i in 0..limit {
        z = z * z + c;
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
    }

    None
}

/// Parse the string `s` as a coordinate pair like `"800x600"` or `"1.0, 0.5"`.
fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(idx) => match (T::from_str(&s[0..idx]), T::from_str(&s[idx + 1..])) {
            (Ok(left), Ok(right)) => Some((left, right)),
            _ => None,
        },
    }
}

/// Parse a pair of floating-point numbers seperated by a comma as a complex number.
fn parse_complex(s: &str) -> Option<Complex64> {
    match parse_pair::<f64>(s, ',') {
        Some((re, im)) => Some(cmplx!(re, im)),
        None => None,
    }
}

/// Given the row and column of a pixel in the output image,
/// return the corresponding point on the complex plane.
fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    upper_l: Complex64,
    lower_r: Complex64,
) -> Complex64 {
    let (w, h) = (lower_r.re - upper_l.re, upper_l.im - lower_r.im);
    let re = upper_l.re + (((pixel.0 as f64) * w) / bounds.0 as f64);
    let im = upper_l.im - (((pixel.1 as f64) * h) / bounds.1 as f64);

    cmplx!(re, im)
}

fn render(pixels: &mut [u8], bounds: (usize, usize), upper_l: Complex64, lower_r: Complex64) {
    assert!(pixels.len() == bounds.0 * bounds.1);
    for row in 0..bounds.1 {
        for col in 0..bounds.0 {
            let point = pixel_to_point(bounds, (col, row), upper_l, lower_r);
            pixels[row * bounds.0 + col] = match escape_time(point, 255) {
                None => 0,
                Some(count) => 255 - count as u8,
            };
        }
    }
}

/// Write the buffer `pixels`,
/// whose dimensions are given by `bounds`, to the file named `filename`.
fn write_image(
    filename: &str,
    pixels: &[u8],
    bounds: (usize, usize),
) -> Result<(), image::ImageError> {
    let output = File::create(filename)?;
    let encoder = PngEncoder::new(output);

    encoder.write_image(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::L8)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::{parse_complex as pc, parse_pair as pp, pixel_to_point as ptp};
    use crate::Complex64;

    #[test]
    fn parse_pair() {
        assert_eq!(pp::<i32>("", ','), None);
        assert_eq!(pp::<u32>("10", 'x'), None);
        assert_eq!(pp::<u32>("10x20", ','), None);
        assert_eq!(pp::<f64>("400.0x", 'x'), None);
        assert_eq!(pp::<u32>("10,20", ','), Some((10u32, 20u32)));
        assert_eq!(pp::<u32>("400x600", 'x'), Some((400u32, 600u32)));
        assert_eq!(pp::<f64>("400.0x600.5", 'x'), Some((400.0f64, 600.5f64)));
    }

    #[test]
    fn parse_complex() {
        assert_eq!(pc("1.25,-0.0625"), Some(cmplx!(1.25, -0.0625)));
        assert_eq!(pc("0.0,0.0"), Some(cmplx!()));
        assert_eq!(pc(",-1.0256"), None);
    }

    #[test]
    fn pixel_to_point() {
        assert_eq!(
            ptp((100, 100), (25, 75), cmplx!(-1.0, 1.0), cmplx!(1.0, -1.0)),
            cmplx!(-0.5, -0.5)
        );
    }
}

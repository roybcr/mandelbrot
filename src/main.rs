use std::str::FromStr;

use num_complex::Complex64;

macro_rules! cmplx {
    () => {
        Complex64::new(0.0, 0.0)
    };
}

fn main() {}

fn _complex_sqr_add_loop(c: Complex64) {
    let mut z = cmplx!();
    loop {
        z = z * z + c;
    }
}

fn _escape_time(c: Complex64, limit: u32) -> Option<u32> {
    let mut z = cmplx!();
    for i in 0..limit {
        z = z * z + c;
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
    }

    None
}

/// Parse the string `s` as a coordinate pair like `"800x600"` or `"1.0,0.5"`.

pub fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(idx) => match (T::from_str(&s[0..idx]), T::from_str(&s[idx + 1..])) {
            (Ok(left), Ok(right)) => Some((left, right)),
            _ => None,
        },
    }
}

#[cfg(test)]
mod test {
    use super::parse_pair as ps;

    #[test]
    fn parse_pair() {
        assert_eq!(ps::<u32>("400x600", 'x'), Some((400u32, 600u32)));
    }
}

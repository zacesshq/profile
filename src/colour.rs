//! Colour maths: sRGB hex parsing, OKLab and OKLCH, WCAG 2.2 contrast, APCA and simulated colour
//! vision deficiencies.
//!
//! OKLab follows Björn Ottosson's published matrices. The WCAG figures follow the relative
//! luminance definition in WCAG 2.2. APCA follows the APCA-W3 0.0.98G-4g constants. The colour
//! vision simulation uses the Machado, Oliveira and Fernandes (2009) matrices at full severity,
//! the same ones the Zacess themes check with.

use std::fmt;
use std::str::FromStr;

/// An sRGB colour with 8 bits per channel, the form every target file stores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// A colour in OKLab.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

/// A colour in OKLCH. Hue is in degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklch {
    pub l: f64,
    pub c: f64,
    pub h: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseHexError(pub String);

impl fmt::Display for ParseHexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\"{}\" is not a colour: write it as #rrggbb, such as #1a2b3c",
            self.0
        )
    }
}

impl std::error::Error for ParseHexError {}

impl Rgb {
    pub const BLACK: Rgb = Rgb::new(0, 0, 0);
    pub const WHITE: Rgb = Rgb::new(255, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Rgb { r, g, b }
    }

    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Hex digits without the leading hash, for formats that refuse it.
    pub fn hex_bare(self) -> String {
        format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// The xterm `rgb:rr/gg/bb` form used by OSC 4, 10 and 11.
    pub fn xparse(self) -> String {
        format!("rgb:{:02x}/{:02x}/{:02x}", self.r, self.g, self.b)
    }

    fn channels(self) -> [f64; 3] {
        [
            f64::from(self.r) / 255.0,
            f64::from(self.g) / 255.0,
            f64::from(self.b) / 255.0,
        ]
    }

    pub fn linear(self) -> [f64; 3] {
        self.channels().map(srgb_to_linear)
    }

    /// WCAG 2.2 relative luminance, 0 for black and 1 for white.
    pub fn relative_luminance(self) -> f64 {
        let [r, g, b] = self.linear();
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    pub fn to_oklab(self) -> Oklab {
        linear_to_oklab(self.linear())
    }

    pub fn to_oklch(self) -> Oklch {
        self.to_oklab().to_oklch()
    }

    /// The nearest 8 bit colour to a linear RGB triple, clamping each channel into the gamut.
    pub fn from_linear(rgb: [f64; 3]) -> Rgb {
        let to_byte = |c: f64| (linear_to_srgb(c.clamp(0.0, 1.0)) * 255.0).round() as u8;
        Rgb::new(to_byte(rgb[0]), to_byte(rgb[1]), to_byte(rgb[2]))
    }

    /// The cube root of relative luminance: the OKLab lightness of the grey with the same
    /// luminance, which is all a viewer with no colour vision has to go on.
    pub fn tone(self) -> f64 {
        self.relative_luminance().cbrt()
    }
}

/// A colour vision deficiency to simulate. Each one is the full form, the hardest case of its kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deficiency {
    Protanopia,
    Deuteranopia,
    Tritanopia,
}

impl Deficiency {
    pub const ALL: [Deficiency; 3] = [
        Deficiency::Protanopia,
        Deficiency::Deuteranopia,
        Deficiency::Tritanopia,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Deficiency::Protanopia => "protanopia",
            Deficiency::Deuteranopia => "deuteranopia",
            Deficiency::Tritanopia => "tritanopia",
        }
    }

    /// Machado, Oliveira and Fernandes (2009), severity 1.0, applied to linear RGB.
    fn matrix(self) -> [[f64; 3]; 3] {
        match self {
            Deficiency::Protanopia => [
                [0.152_286, 1.052_583, -0.204_868],
                [0.114_503, 0.786_281, 0.099_216],
                [-0.003_882, -0.048_116, 1.051_998],
            ],
            Deficiency::Deuteranopia => [
                [0.367_322, 0.860_646, -0.227_968],
                [0.280_085, 0.672_501, 0.047_413],
                [-0.011_82, 0.042_94, 0.968_881],
            ],
            Deficiency::Tritanopia => [
                [1.255_528, -0.076_749, -0.178_779],
                [-0.078_411, 0.930_809, 0.147_602],
                [0.004_733, 0.691_367, 0.303_9],
            ],
        }
    }
}

impl fmt::Display for Deficiency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// How a colour looks to a person with the given deficiency, rounded to 8 bits as the Zacess themes
/// do, so a distance measured here is the distance measured there.
pub fn simulate(colour: Rgb, deficiency: Deficiency) -> Rgb {
    let rgb = colour.linear();
    let m = deficiency.matrix();
    Rgb::from_linear(m.map(|row| row[0] * rgb[0] + row[1] * rgb[1] + row[2] * rgb[2]))
}

/// The OKLab distance between two colours as a person with the given deficiency sees them.
pub fn simulated_distance(a: Rgb, b: Rgb, deficiency: Deficiency) -> f64 {
    simulate(a, deficiency)
        .to_oklab()
        .distance(simulate(b, deficiency).to_oklab())
}

impl FromStr for Rgb {
    type Err = ParseHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || ParseHexError(s.to_string());
        let digits = s.strip_prefix('#').ok_or_else(err)?;
        if digits.len() != 6 || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(err());
        }
        let byte = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).map_err(|_| err());
        Ok(Rgb::new(byte(0)?, byte(2)?, byte(4)?))
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.hex())
    }
}

impl serde::Serialize for Rgb {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.hex())
    }
}

impl<'de> serde::Deserialize<'de> for Rgb {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

pub fn srgb_to_linear(c: f64) -> f64 {
    // 0.04045 is the corrected threshold WCAG 2.2 uses; the older 0.03928 differs only below 1/255
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

pub fn linear_to_oklab([r, g, b]: [f64; 3]) -> Oklab {
    let l = 0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b;
    let m = 0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b;
    let s = 0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b;
    let (l, m, s) = (l.cbrt(), m.cbrt(), s.cbrt());
    Oklab {
        l: 0.210_454_255_3 * l + 0.793_617_785 * m - 0.004_072_046_8 * s,
        a: 1.977_998_495_1 * l - 2.428_592_205 * m + 0.450_593_709_9 * s,
        b: 0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766 * s,
    }
}

impl Oklab {
    pub fn to_linear(self) -> [f64; 3] {
        let l = self.l + 0.396_337_777_4 * self.a + 0.215_803_757_3 * self.b;
        let m = self.l - 0.105_561_345_8 * self.a - 0.063_854_172_8 * self.b;
        let s = self.l - 0.089_484_177_5 * self.a - 1.291_485_548 * self.b;
        let (l, m, s) = (l.powi(3), m.powi(3), s.powi(3));
        [
            4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
            -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
            -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701 * s,
        ]
    }

    pub fn to_oklch(self) -> Oklch {
        let c = self.a.hypot(self.b);
        let h = self.b.atan2(self.a).to_degrees().rem_euclid(360.0);
        Oklch { l: self.l, c, h }
    }

    pub fn distance(self, other: Oklab) -> f64 {
        ((self.l - other.l).powi(2) + (self.a - other.a).powi(2) + (self.b - other.b).powi(2))
            .sqrt()
    }
}

impl Oklch {
    pub fn new(l: f64, c: f64, h: f64) -> Self {
        Oklch { l, c, h }
    }

    pub fn to_oklab(self) -> Oklab {
        let h = self.h.to_radians();
        Oklab {
            l: self.l,
            a: self.c * h.cos(),
            b: self.c * h.sin(),
        }
    }

    /// Whether the colour is inside the sRGB gamut, with a small tolerance for rounding.
    pub fn in_gamut(self) -> bool {
        self.to_oklab()
            .to_linear()
            .iter()
            .all(|&c| (-1e-6..=1.0 + 1e-6).contains(&c))
    }

    /// Converts to sRGB, lowering chroma until the colour fits the gamut. Lightness and hue are
    /// kept, so the result still looks like the colour that was asked for.
    pub fn to_rgb(self) -> Rgb {
        let l = self.l.clamp(0.0, 1.0);
        let mut colour = Oklch::new(l, self.c.max(0.0), self.h);
        if !colour.in_gamut() {
            let (mut lo, mut hi) = (0.0, colour.c);
            for _ in 0..32 {
                let mid = (lo + hi) / 2.0;
                if Oklch::new(l, mid, self.h).in_gamut() {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            colour.c = lo;
        }
        Rgb::from_linear(colour.to_oklab().to_linear())
    }
}

/// The WCAG 2.2 contrast ratio between two colours, from 1 to 21. Order does not matter.
pub fn contrast_ratio(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (a.relative_luminance(), b.relative_luminance());
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// APCA lightness contrast (Lc) of text on a background, APCA-W3 0.0.98G-4g. Positive for dark
/// text on a light background and negative for light text on a dark one.
pub fn apca_lc(text: Rgb, background: Rgb) -> f64 {
    // APCA uses a plain 2.4 power curve rather than the piecewise sRGB curve, on purpose
    let y = |c: Rgb| {
        let [r, g, b] = c.channels().map(|v| v.powf(2.4));
        0.212_672_9 * r + 0.715_152_2 * g + 0.072_175 * b
    };
    let clamp_black = |y: f64| {
        if y < 0.022 {
            y + (0.022 - y).powf(1.414)
        } else {
            y
        }
    };
    let (yt, yb) = (clamp_black(y(text)), clamp_black(y(background)));
    if (yb - yt).abs() < 0.0005 {
        return 0.0;
    }
    let sapc = if yb > yt {
        let s = (yb.powf(0.56) - yt.powf(0.57)) * 1.14;
        if s < 0.1 { 0.0 } else { s - 0.027 }
    } else {
        let s = (yb.powf(0.65) - yt.powf(0.62)) * 1.14;
        if s > -0.1 { 0.0 } else { s + 0.027 }
    };
    sapc * 100.0
}

/// Rounds a ratio down to two decimals, so a printed figure never overstates the real one.
pub fn floor2(x: f64) -> f64 {
    (x * 100.0).floor() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(s: &str) -> Rgb {
        s.parse().unwrap()
    }

    #[test]
    fn black_on_white_is_21_to_1() {
        assert!((contrast_ratio(Rgb::BLACK, Rgb::WHITE) - 21.0).abs() < 1e-9);
        assert!((contrast_ratio(Rgb::WHITE, Rgb::BLACK) - 21.0).abs() < 1e-9);
    }

    #[test]
    fn same_colour_is_1_to_1() {
        assert!((contrast_ratio(rgb("#777777"), rgb("#777777")) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn known_wcag_ratios() {
        // the classic AA boundary: #767676 passes 4.5:1 on white and #777777 just misses it
        assert_eq!(floor2(contrast_ratio(rgb("#767676"), Rgb::WHITE)), 4.54);
        assert_eq!(floor2(contrast_ratio(rgb("#777777"), Rgb::WHITE)), 4.47);
        // the AAA boundary on white
        assert_eq!(floor2(contrast_ratio(rgb("#595959"), Rgb::WHITE)), 7.0);
        // pure primaries on black and white
        assert_eq!(floor2(contrast_ratio(rgb("#ff0000"), Rgb::WHITE)), 3.99);
        assert_eq!(floor2(contrast_ratio(rgb("#0000ff"), Rgb::WHITE)), 8.59);
        assert_eq!(floor2(contrast_ratio(rgb("#00ff00"), Rgb::BLACK)), 15.3);
        // the Zacess Light text pair, published as 18.8:1 rounded down
        assert_eq!(floor2(contrast_ratio(rgb("#111111"), Rgb::WHITE)), 18.88);
    }

    #[test]
    fn relative_luminance_endpoints() {
        assert_eq!(Rgb::BLACK.relative_luminance(), 0.0);
        assert!((Rgb::WHITE.relative_luminance() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn hex_round_trip_and_errors() {
        let c = rgb("#1A2b3C");
        assert_eq!(c, Rgb::new(0x1a, 0x2b, 0x3c));
        assert_eq!(c.hex(), "#1a2b3c");
        assert_eq!(c.xparse(), "rgb:1a/2b/3c");
        for bad in ["1a2b3c", "#1a2b3", "#1a2b3cd", "#gg0000", ""] {
            assert!(bad.parse::<Rgb>().is_err(), "{bad} should not parse");
        }
    }

    #[test]
    fn oklab_matches_published_values() {
        let white = Rgb::WHITE.to_oklab();
        assert!((white.l - 1.0).abs() < 1e-4 && white.a.abs() < 1e-4 && white.b.abs() < 1e-4);
        let red = rgb("#ff0000").to_oklab();
        assert!((red.l - 0.627_96).abs() < 1e-4);
        assert!((red.a - 0.224_86).abs() < 1e-4);
        assert!((red.b - 0.125_85).abs() < 1e-4);
        let blue = rgb("#0000ff").to_oklch();
        assert!((blue.l - 0.452_01).abs() < 1e-4);
        assert!((blue.h - 264.05).abs() < 0.1);
    }

    #[test]
    fn oklch_round_trip_is_exact_for_every_sample() {
        for hex in [
            "#000000", "#ffffff", "#ffd400", "#0049a3", "#97016b", "#80c201", "#fdf6e3",
        ] {
            let c = rgb(hex);
            assert_eq!(c.to_oklch().to_rgb(), c, "{hex}");
        }
    }

    #[test]
    fn out_of_gamut_keeps_lightness_and_hue() {
        let wild = Oklch::new(0.7, 0.5, 30.0);
        assert!(!wild.in_gamut());
        let mapped = wild.to_rgb().to_oklch();
        assert!((mapped.l - 0.7).abs() < 0.01);
        assert!((mapped.h - 30.0).abs() < 2.0);
    }

    #[test]
    fn apca_known_values() {
        // APCA's own reference figures: black on white Lc 106.04, white on black Lc -107.88
        assert!((apca_lc(Rgb::BLACK, Rgb::WHITE) - 106.04).abs() < 0.01);
        assert!((apca_lc(Rgb::WHITE, Rgb::BLACK) + 107.88).abs() < 0.01);
        // #888888 text on white, Lc 63.06 in the reference implementation
        assert!((apca_lc(rgb("#888888"), Rgb::WHITE) - 63.06).abs() < 0.05);
        assert_eq!(apca_lc(rgb("#777777"), rgb("#777777")), 0.0);
    }

    #[test]
    fn simulation_matches_the_published_matrices() {
        // greys have equal channels and every Machado row sums to one, so they are unchanged
        for hex in ["#000000", "#ffffff", "#777777"] {
            for d in Deficiency::ALL {
                assert_eq!(simulate(rgb(hex), d), rgb(hex), "{hex} {d}");
            }
        }
        // pure red under protanopia keeps little more than its luminance, a dark olive
        let red = simulate(rgb("#ff0000"), Deficiency::Protanopia);
        assert!(red.r < 0x80 && red.g < 0x80 && red.b < 0x10, "{red}");
        // red and green that differ only in hue collapse under deuteranopia but not in normal vision
        let (a, b) = (rgb("#b35a00"), rgb("#5a7a00"));
        assert!(a.to_oklab().distance(b.to_oklab()) > 0.1);
        assert!(simulated_distance(a, b, Deficiency::Deuteranopia) < 0.06);
    }

    #[test]
    fn tone_is_the_cube_root_of_luminance() {
        assert_eq!(Rgb::BLACK.tone(), 0.0);
        assert!((Rgb::WHITE.tone() - 1.0).abs() < 1e-12);
        let grey = rgb("#777777");
        assert!((grey.tone() - grey.to_oklab().l).abs() < 1e-3);
    }
}

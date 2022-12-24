use std::fmt;

#[derive(Debug, Clone)]
pub enum WebColor {
    RGB(RGBColor),
    Named(NamedColor),
}

#[derive(Debug, Clone)]
pub struct RGBColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RGBColor {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

impl fmt::Display for RGBColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NamedColor {
    White,
    Black,
}

impl fmt::Display for NamedColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NamedColor::White => write!(f, "white"),
            NamedColor::Black => write!(f, "black"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_color() {
        let color = RGBColor::new(255, 255, 255);
        assert_eq!(color.to_string(), "#FFFFFF");

        let color = RGBColor::new(0, 0, 0);
        assert_eq!(color.to_string(), "#000000");

        let color = RGBColor::new(73, 123, 145);
        assert_eq!(color.to_string(), "#497B91");
    }

    #[test]
    fn named_color() {
        assert_eq!(NamedColor::White.to_string(), "white");
        assert_eq!(NamedColor::Black.to_string(), "black");
    }
}

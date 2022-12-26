use derive_more::Display;

#[derive(Debug, Clone, Display)]
pub enum WebColor {
    #[display(fmt = "{}", _0)]
    RGB(RGBColor),
    #[display(fmt = "{}", _0)]
    Named(NamedColor),
}

impl Default for WebColor {
    fn default() -> Self {
        WebColor::Named(NamedColor::Black)
    }
}

#[derive(Debug, Clone, Default, Display)]
#[display(fmt = "#{:02X}{:02X}{:02X}", red, green, blue)]
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

#[derive(Debug, Clone, Copy, Display)]
pub enum NamedColor {
    #[display(fmt = "white")]
    White,
    #[display(fmt = "black")]
    Black,
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

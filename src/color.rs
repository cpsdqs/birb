use swift_birb::protocol::SBColor;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Into<SBColor> for Color {
    fn into(self) -> SBColor {
        SBColor {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}

pub enum GdkInterpType {
    Nearest,
    Tiles,
    Bilinear,
    Hyper
}

impl GdkInterpType {
    pub fn to_i32(&self) -> i32 {
        use self::GdkInterpType::*;

        match self {
            &Nearest => 0,
            &Tiles => 1,
            &Bilinear => 2,
            &Hyper => 3,
        }
    }
}


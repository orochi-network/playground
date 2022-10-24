pub trait PNumericEncoding {
    // transform into u32 value
    fn to_u32(&self) -> u32;

    // from u32 transforming into Self
    fn from_u32(v: u32) -> Self;
}
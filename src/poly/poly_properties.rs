use serde::Serialize;

#[derive(Clone, Debug, serde::Serialize)]
pub struct NumberRange {
    pub min: f32,
    pub max: f32
}

#[derive(Clone, Debug, serde_tuple::Serialize_tuple)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32
}

#[derive(Clone, Debug, serde_tuple::Serialize_tuple)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

#[derive(Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let converted_r = format!("{:02x}", if self.r == 1.0 { 255.0 } else { (self.r * 256.0).floor() } as u32);
        let converted_g = format!("{:02x}", if self.g == 1.0 { 255.0 } else { (self.g * 256.0).floor() } as u32);
        let converted_b = format!("{:02x}", if self.b == 1.0 { 255.0 } else { (self.b * 256.0).floor() } as u32);
        let converted_a = format!("{:02x}", if self.a == 1.0 { 255.0 } else { (self.a * 256.0).floor() } as u32);
        let state = serializer.serialize_str(&(converted_r + &converted_g + &converted_b + &converted_a));
        state
    }
}
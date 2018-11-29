use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign};

#[derive(Copy, Clone)]
pub struct Rect {
    min: float2,
    max: float2,
}

impl Rect {
    pub fn new(min: float2, max: float2) -> Self {
        Rect {
            min,
            max,
        }
    }

    pub fn contains(&self, point: float2) -> bool {
        (self.min.0 < point.0 && self.min.1 < point.1) ||
        (self.max.0 > point.0 && self.max.1 > point.1)
    }

    pub fn outside(&self, other: Rect) -> bool {
        unimplemented!()
    }

    pub fn clip(&self, other: Rect) -> Option<Rect> {
        unimplemented!()
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub struct float4(pub f32, pub f32, pub f32, pub f32);

impl Add for float4 {
    type Output = float4;

    fn add(self, rhs: float4) -> float4 {
        float4(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2, self.3 + rhs.3)
    }
}

impl AddAssign for float4 {
    fn add_assign(&mut self, rhs: float4) {
        *self = *self + rhs;
    }
}

impl Sub for float4 {
    type Output = float4;

    fn sub(self, rhs: float4) -> float4 {
        float4(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2, self.3 - rhs.3)
    }
}

impl SubAssign for float4 {
    fn sub_assign(&mut self, rhs: float4) {
        *self = *self - rhs;
    }
}

impl Mul for float4 {
    type Output = float4;

    fn mul(self, rhs: float4) -> float4 {
        float4(self.0 * rhs.0, self.1 * rhs.1, self.2 * rhs.2, self.3 * rhs.3)
    }
}

impl MulAssign for float4 {
    fn mul_assign(&mut self, rhs: float4) {
        *self = *self * rhs;
    }
}

impl Div for float4 {
    type Output = float4;

    fn div(self, rhs: float4) -> float4 {
        float4(self.0 / rhs.0, self.1 / rhs.1, self.2 / rhs.2, self.3 / rhs.3)
    }
}

impl DivAssign for float4 {
    fn div_assign(&mut self, rhs: float4) {
        *self = *self / rhs;
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub struct float2(pub f32, pub f32);

impl float2 {
    pub fn length(&self) -> f32 {
        (self.0 * self.0 + self.1 * self.1).sqrt()
    }
}

impl Add for float2 {
    type Output = float2;

    fn add(self, rhs: float2) -> float2 {
        float2(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl AddAssign for float2 {
    fn add_assign(&mut self, rhs: float2) {
        *self = *self + rhs;
    }
}

impl Sub for float2 {
    type Output = float2;

    fn sub(self, rhs: float2) -> float2 {
        float2(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl SubAssign for float2 {
    fn sub_assign(&mut self, rhs: float2) {
        *self = *self - rhs;
    }
}

impl Mul for float2 {
    type Output = float2;

    fn mul(self, rhs: float2) -> float2 {
        float2(self.0 * rhs.0, self.1 * rhs.1)
    }
}

impl Mul<f32> for float2 {
    type Output = float2;

    fn mul(self, rhs: f32) -> float2 {
        float2(self.0 * rhs, self.1 * rhs)
    }
}

impl MulAssign for float2 {
    fn mul_assign(&mut self, rhs: float2) {
        *self = *self * rhs;
    }
}

impl Div for float2 {
    type Output = float2;

    fn div(self, rhs: float2) -> float2 {
        float2(self.0 / rhs.0, self.1 / rhs.1)
    }
}

impl DivAssign for float2 {
    fn div_assign(&mut self, rhs: float2) {
        *self = *self / rhs;
    }
}

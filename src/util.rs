use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Vec3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl Vec3 {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn volume(&self) -> usize {
        self.x.unsigned_abs() as usize
            * self.y.unsigned_abs() as usize
            * self.z.unsigned_abs() as usize
    }

    pub fn abs(&self) -> UVec3 {
        UVec3 {
            x: self.x.unsigned_abs(),
            y: self.y.unsigned_abs(),
            z: self.z.unsigned_abs(),
        }
    }

    pub fn signum(&self) -> Vec3 {
        Vec3 {
            x: self.x.signum(),
            y: self.y.signum(),
            z: self.z.signum(),
        }
    }

    pub fn volume_to(&self, other: &Vec3) -> u32 {
        self.size_to(other).volume()
    }

    pub fn size_to(&self, other: &Vec3) -> UVec3 {
        UVec3 {
            x: (self.x - other.x).unsigned_abs() + 1,
            y: (self.y - other.y).unsigned_abs() + 1,
            z: (self.z - other.z).unsigned_abs() + 1,
        }
    }
}

impl Add<Vec3> for Vec3 {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add<UVec3> for Vec3 {
    type Output = Self;

    fn add(self, rhs: UVec3) -> Self::Output {
        Self {
            x: self.x + rhs.x as i32,
            y: self.y + rhs.y as i32,
            z: self.z + rhs.z as i32,
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct UVec3 {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}
impl UVec3 {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    pub fn volume(&self) -> u32 {
        self.x * self.y * self.z
    }
}

macro_rules! vec_debug {
    ($type:ty) => {
        impl std::fmt::Debug for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "({}, {}, {})", self.x, self.y, self.z)
            }
        }
    };
}
vec_debug!(Vec3);
vec_debug!(UVec3);

#[cfg(feature = "chrono")]
pub(crate) fn current_time() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}
#[cfg(all(not(target_family = "wasm"), not(feature = "chrono")))]
pub(crate) fn current_time() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}
#[cfg(all(target_family = "wasm", not(feature = "chrono")))]
pub(crate) fn current_time() -> i64 {
    js_sys::Date::now() as i64
}

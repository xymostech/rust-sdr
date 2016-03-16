use std::cmp;
use std::ops::{Sub, Add, Mul};

#[derive(Clone, Copy)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T: Copy + Ord> Vec2<T> {
    pub fn clamp(&self, min: &Vec2<T>, max: &Vec2<T>) -> Vec2<T> {
        Vec2::<T> {
            x: cmp::max(min.x, cmp::min(max.x, self.x)),
            y: cmp::max(min.y, cmp::min(max.y, self.y)),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Copy> Vec3<T> {
    // Swizzle
    pub fn xy(&self) -> Vec2<T> {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}

impl<T> Sub<Vec3<T>> for Vec3<T>
        where T: Sub<T, Output = T> {
    type Output = Vec3<T>;

    fn sub(self, other: Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl<T: Copy + Mul<T, Output = T> + Sub<T, Output = T>> Vec3<T> {
    pub fn cross(self, other: Vec3<T>) -> Vec3<T> {
        Vec3::<T> {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

impl<T: Copy + Mul<T, Output = T> + Add<T, Output = T>> Vec3<T> {
    pub fn dot(self, other: Vec3<T>) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl Vec3<f32> {
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn norm(self) -> Vec3<f32> {
        let length = self.length();

        Vec3 {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }
}

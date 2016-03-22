use vec::{Vec3, Vec4};

use num::traits::{Num, Zero};
use std::ops::{Sub, Add, Mul};
use std::fmt::Display;

pub struct Matrix4x4<T> {
    data: Vec<T>,
}

impl<T: Copy + Display> Matrix4x4<T> {
    pub fn print(&self) {
        for i in 0..4 {
            println!("{} {} {} {}", self.get(i, 0), self.get(i, 1), self.get(i, 2), self.get(i, 3));
        }
    }
}

impl Matrix4x4<f32> {
    pub fn translation(by: Vec3<f32>) -> Matrix4x4<f32> {
        Matrix4x4 {
            data: vec![
                1.0,  0.0,  0.0,  0.0,
                0.0,  1.0,  0.0,  0.0,
                0.0,  0.0,  1.0,  0.0,
                by.x, by.y, by.z, 1.0,
            ],
        }
    }

    pub fn rotation(theta: f32, about: Vec3<f32>) -> Matrix4x4<f32> {
        let about = about.norm();

        let u = about.x;
        let v = about.y;
        let w = about.z;

        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        Matrix4x4 {
            data: vec![
                u * u + (v * v + w * w) * cos_theta, u * v * (1.0 - cos_theta) + w * sin_theta,
                    u * w * (1.0 - cos_theta) - v * sin_theta, 0.0,
                u * v * (1.0 - cos_theta) - w * sin_theta, v * v + (u * u + w * w) * cos_theta,
                    v * w * (1.0 - cos_theta) + u * sin_theta, 0.0,
                u * w * (1.0 - cos_theta) + v * sin_theta, v * w * (1.0 - cos_theta) - u * sin_theta,
                    w * w + (u * u + v * v) * cos_theta, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn scale(by: Vec3<f32>) -> Matrix4x4<f32> {
        Matrix4x4 {
            data: vec![
                by.x, 0.0,  0.0,  0.0,
                0.0,  by.y, 0.0,  0.0,
                0.0,  0.0,  by.z, 0.0,
                0.0,  0.0,  0.0,  1.0,
            ],
        }
    }

    pub fn lookat(eye: Vec3<f32>, center: Vec3<f32>, up: Vec3<f32>) -> Matrix4x4<f32> {
        let z = (eye - center).norm();
        let x = up.cross(z).norm();
        let y = z.cross(x).norm();

        let mut change_of_frame = Matrix4x4::identity();
        let mut translate = Matrix4x4::identity();

        change_of_frame.set(0, 0, x.x);
        change_of_frame.set(0, 1, x.y);
        change_of_frame.set(0, 2, x.z);

        change_of_frame.set(1, 0, y.x);
        change_of_frame.set(1, 1, y.y);
        change_of_frame.set(1, 2, y.z);

        change_of_frame.set(2, 0, z.x);
        change_of_frame.set(2, 1, z.y);
        change_of_frame.set(2, 2, z.z);

        translate.set(0, 3, -center.x);
        translate.set(1, 3, -center.y);
        translate.set(2, 3, -center.z);

        change_of_frame * translate
    }

    pub fn viewport(x: f32, y: f32, width: f32, height: f32, depth: f32) -> Matrix4x4<f32> {
        Matrix4x4 {
            data: vec![
                width / 2.0, 0.0, 0.0, 0.0,
                0.0, height / 2.0, 0.0, 0.0,
                0.0, 0.0, depth / 2.0, 0.0,
                x + width / 2.0, y + height / 2.0, depth / 2.0, 1.0,
            ],
        }
    }

    pub fn perspective(dist: f32) -> Matrix4x4<f32> {
        Matrix4x4 {
            data: vec![
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, -1.0 / dist,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }
}

impl<T: Copy> Matrix4x4<T> {
    fn index(&self, row: usize, col: usize) -> usize {
        col * 4 + row
    }

    fn set(&mut self, row: usize, col: usize, val: T) {
        let index = self.index(row, col);
        self.data[index] = val;
    }

    fn get(&self, row: usize, col: usize) -> T {
        self.data[self.index(row, col)]
    }
}

impl<T: Copy + Num> Matrix4x4<T> {
    pub fn new() -> Matrix4x4<T> {
        let mut data: Vec<T> = Vec::with_capacity(4 * 4);
        data.resize(4 * 4, T::zero());

        Matrix4x4::<T> {
            data: data,
        }
    }

    pub fn identity() -> Matrix4x4<T> {
        let mut mat = Matrix4x4::new();

        for i in 0..4 {
            mat.set(i, i, T::one());
        }

        mat
    }
}

impl<T> Mul<Matrix4x4<T>> for Matrix4x4<T>
        where T: Mul<T, Output=T> + Add<T, Output=T> + Copy + Num {
    type Output = Matrix4x4<T>;

    fn mul(self, rhs: Matrix4x4<T>) -> Matrix4x4<T> {
        //assert!(self.cols == rhs.rows, "Multiplied matrices must have compatible dimensions");

        let mut result = Matrix4x4::new();

        for i in 0..4 {
            for j in 0..4 {
                let mut val: T = T::zero();
                for k in 0..4 {
                    val = val + self.get(i, k) * rhs.get(k, j);
                }
                result.set(i, j, val);
            }
        }

        result
    }
}

impl<T> Mul<Vec4<T>> for Matrix4x4<T>
        where T: Mul<T, Output=T> + Add<T, Output=T> + Copy + Num {
    type Output = Vec4<T>;

    fn mul(self, rhs: Vec4<T>) -> Vec4<T> {
        Vec4 {
            x: self.get(0, 0) * rhs.x + self.get(0, 1) * rhs.y + self.get(0, 2) * rhs.z + self.get(0, 3) * rhs.w,
            y: self.get(1, 0) * rhs.x + self.get(1, 1) * rhs.y + self.get(1, 2) * rhs.z + self.get(1, 3) * rhs.w,
            z: self.get(2, 0) * rhs.x + self.get(2, 1) * rhs.y + self.get(2, 2) * rhs.z + self.get(2, 3) * rhs.w,
            w: self.get(3, 0) * rhs.x + self.get(3, 1) * rhs.y + self.get(3, 2) * rhs.z + self.get(3, 3) * rhs.w,
        }
    }
}

impl<'a, 'b, T> Mul<&'a Vec4<T>> for &'b Matrix4x4<T>
        where T: Mul<T, Output=T> + Add<T, Output=T> + Copy + Num {
    type Output = Vec4<T>;

    fn mul(self, rhs: &Vec4<T>) -> Vec4<T> {
        Vec4 {
            x: self.get(0, 0) * rhs.x + self.get(0, 1) * rhs.y + self.get(0, 2) * rhs.z + self.get(0, 3) * rhs.w,
            y: self.get(1, 0) * rhs.x + self.get(1, 1) * rhs.y + self.get(1, 2) * rhs.z + self.get(1, 3) * rhs.w,
            z: self.get(2, 0) * rhs.x + self.get(2, 1) * rhs.y + self.get(2, 2) * rhs.z + self.get(2, 3) * rhs.w,
            w: self.get(3, 0) * rhs.x + self.get(3, 1) * rhs.y + self.get(3, 2) * rhs.z + self.get(3, 3) * rhs.w,
        }
    }
}

impl<T> Add<Matrix4x4<T>> for Matrix4x4<T>
        where T: Add<T, Output=T> + Copy + Num {
    type Output = Matrix4x4<T>;

    fn add(self, rhs: Matrix4x4<T>) -> Matrix4x4<T> {
        //assert!(self.rows == rhs.rows, "Added matrices must have compatible dimensions");
        //assert!(self.cols == rhs.cols, "Added matrices must have compatible dimensions");

        let mut result = Matrix4x4::new();

        for i in 0..4 {
            for j in 0..4 {
                result.set(i, j, self.get(i, j) + rhs.get(i, j));
            }
        }

        result
    }
}

impl<T> Sub<Matrix4x4<T>> for Matrix4x4<T>
        where T: Sub<T, Output=T> + Copy + Num {
    type Output = Matrix4x4<T>;

    fn sub(self, rhs: Matrix4x4<T>) -> Matrix4x4<T> {
        //assert!(self.rows == rhs.rows, "Subtracted matrices must have compatible dimensions");
        //assert!(self.cols == rhs.cols, "Subtracted matrices must have compatible dimensions");

        let mut result = Matrix4x4::new();

        for i in 0..4 {
            for j in 0..4 {
                result.set(i, j, self.get(i, j) - rhs.get(i, j));
            }
        }

        result
    }
}

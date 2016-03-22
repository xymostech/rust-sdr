#![allow(dead_code)]
extern crate imagefmt;
extern crate num;
extern crate rand;

mod vec;
mod image;
mod obj;
mod shader;
mod matrix;

use vec::{Vec2, Vec3, Vec4};
use image::*;
use shader::{Vary, Shader, draw_triangle};
use matrix::*;
use obj::*;

//use std::f32;

#[derive(Clone, Copy)]
struct Vars {
    normal: Vec3<f32>,
    tex: Vec2<f32>,
}

impl Vary for Vars {
    fn vary(v1: &Vars, v2: &Vars, v3: &Vars, bary: Vec3<f32>) -> Vars {
        Vars {
            normal: (v1.normal * bary.x + v2.normal * bary.y + v3.normal * bary.z).norm(),
            tex: v1.tex * bary.x + v2.tex * bary.y + v3.tex * bary.z,
        }
    }
}

struct MyShader<'a> {
    mat: &'a Matrix4x4<f32>,
    tex: &'a Image,
    light_dir: Vec3<f32>,
}

impl<'a> Shader<Vars> for MyShader<'a> {
    fn vertex(&self, pt: Vec3<f32>, vars: &Vars) -> (Vec4<f32>, Vars) {
        let pt4 = Vec4 { x: pt.x, y: pt.y, z: pt.z, w: 1.0 };

        (
            self.mat * &pt4,
            *vars
        )
    }

    fn fragment(&self, _: Vec2<isize>, vars: Vars) -> Option<Color> {
        let comp = vars.normal.dot(self.light_dir) * 255.0;

        if comp > 0.0 {
            let tex_x = (self.tex.width as f32 * vars.tex.x) as usize;
            let tex_y = (self.tex.height as f32 * vars.tex.y) as usize;
            let tex = self.tex.get_pixel(tex_x, tex_y);
            let shading = Color(comp as u8, comp as u8, comp as u8);

            Some(tex.multiply(&shading))
        } else {
            None
        }
    }
}

fn main() {
    let mut image = Image::new(800, 800);

    let viewport = Matrix4x4::viewport(0.0, 0.0, 800.0, 800.0, 255.0);
    let view = Matrix4x4::scale(Vec3 { x: 0.8, y: 0.8, z: 0.8 });

    let lookat = Matrix4x4::lookat(
        Vec3 { x: 1.0, y: 1.0, z: 3.0 },
        Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        Vec3 { x: 0.0, y: 1.0, z: 0.0 },
    );
    let perspective = Matrix4x4::perspective(3.0);

    let obj = Obj::from_file("head.obj").unwrap();
    let tex = Image::from("head_tex.tga");

    let mat = viewport * perspective * lookat * view;
    let shader = MyShader {
        mat: &mat,
        light_dir: Vec3 { x: 0.0, y: 0.0, z: 1.0 },
        tex: &tex,
    };

    for face in obj.faces.iter() {
        let fp0 = obj.vert(face.0.vindex);
        let fp1 = obj.vert(face.1.vindex);
        let fp2 = obj.vert(face.2.vindex);

        let vars0 = Vars {
            normal: obj.norm_vert(face.0.nindex),
            tex: obj.tex_vert(face.0.tindex),
        };
        let vars1 = Vars {
            normal: obj.norm_vert(face.1.nindex),
            tex: obj.tex_vert(face.1.tindex),
        };
        let vars2 = Vars {
            normal: obj.norm_vert(face.2.nindex),
            tex: obj.tex_vert(face.2.tindex),
        };

        draw_triangle(&vec![(fp0, vars0), (fp1, vars1), (fp2, vars2)], &shader, &mut image);
    }

    image.write("out.tga").unwrap();
}

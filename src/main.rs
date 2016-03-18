#![allow(dead_code)]
extern crate imagefmt;

mod vec;
mod image;
mod obj;
mod shader;

use vec::{Vec2, Vec3, Vec4};
use image::*;
use shader::{Shader, draw_triangle};

struct MyShader<'a> {
    frag: &'a Fn(isize, isize) -> Color,
}

impl<'a> Shader for MyShader<'a> {
    fn vertex(&self, pt: Vec3<f32>) -> (Vec4<f32>, Vec3<f32>) {
        (
            Vec4 {
                x: pt.x,
                y: pt.y,
                z: 0.0,
                w: 1.0,
            },
            Vec3 {
                x: 255.0,
                y: 0.0,
                z: 0.0,
            }
        )
    }

    fn fragment(&self, pt: Vec2<isize>, _: Vec3<f32>) -> Option<Color> {
        Some((self.frag)(pt.x, pt.y))
    }
}

fn make_square(x: f32, y: f32) -> (Vec<Vec3<f32>>, Vec<Vec3<f32>>) {
    (
        vec![
            Vec3 { x: x, y: y, z: 0.0 },
            Vec3 { x: x + 256.0, y: y, z: 0.0 },
            Vec3 { x: x + 256.0, y: y + 256.0, z: 0.0 },
        ],
        vec![
            Vec3 { x: x, y: y, z: 0.0 },
            Vec3 { x: x, y: y + 256.0, z: 0.0 },
            Vec3 { x: x + 256.0, y: y + 256.0, z: 0.0 },
        ],
    )
}

fn main() {
    let mut image = Image::new(256 * 4, 256 * 3);

    {
        let shader = MyShader {
            frag: &|x, y| Color(x as u8, y as u8, 0),
        };

        let (tri1, tri2) = make_square(256.0, 256.0);

        draw_triangle(&tri1, &shader, &mut image);
        draw_triangle(&tri2, &shader, &mut image);
    }

    {
        let shader = MyShader {
            frag: &|x, y| Color(255, y as u8, x as u8),
        };

        let (tri1, tri2) = make_square(512.0, 256.0);

        draw_triangle(&tri1, &shader, &mut image);
        draw_triangle(&tri2, &shader, &mut image);
    }

    {
        let shader = MyShader {
            frag: &|x, y| Color(0, y as u8, (255 - x) as u8),
        };

        let (tri1, tri2) = make_square(0.0, 256.0);

        draw_triangle(&tri1, &shader, &mut image);
        draw_triangle(&tri2, &shader, &mut image);
    }

    {
        let shader = MyShader {
            frag: &|x, y| Color(x as u8, 255, y as u8),
        };

        let (tri1, tri2) = make_square(256.0, 512.0);

        draw_triangle(&tri1, &shader, &mut image);
        draw_triangle(&tri2, &shader, &mut image);
    }

    {
        let shader = MyShader {
            frag: &|x, y| Color(x as u8, 0, (255 - y) as u8),
        };

        let (tri1, tri2) = make_square(256.0, 0.0);

        draw_triangle(&tri1, &shader, &mut image);
        draw_triangle(&tri2, &shader, &mut image);
    }

    {
        let shader = MyShader {
            frag: &|x, y| Color((255 - x) as u8, y as u8, 255),
        };

        let (tri1, tri2) = make_square(768.0, 256.0);

        draw_triangle(&tri1, &shader, &mut image);
        draw_triangle(&tri2, &shader, &mut image);
    }

    image.write("out.tga").unwrap();

    // let obj = Obj::from_file("head.obj").unwrap();
    // let tex = Image::from("head_tex.tga");

    // for face in obj.faces.iter() {
    //     let fp0 = obj.vert(face.0.vindex);
    //     let fp1 = obj.vert(face.1.vindex);
    //     let fp2 = obj.vert(face.2.vindex);

    //     let t0 = Vec2::<isize> { x: ((fp0.x + 1.0) * 400.0) as isize, y: ((fp0.y + 1.0) * 400.0) as isize };
    //     let t1 = Vec2::<isize> { x: ((fp1.x + 1.0) * 400.0) as isize, y: ((fp1.y + 1.0) * 400.0) as isize };
    //     let t2 = Vec2::<isize> { x: ((fp2.x + 1.0) * 400.0) as isize, y: ((fp2.y + 1.0) * 400.0) as isize };

    //     let tex0 = obj.tex_vert(face.0.tindex);
    //     let tex1 = obj.tex_vert(face.1.tindex);
    //     let tex2 = obj.tex_vert(face.2.tindex);

    //     let depths = Vec3 {
    //         x: fp0.z,
    //         y: fp1.z,
    //         z: fp2.z,
    //     };

    //     let normal = face.normal(&obj);

    //     let light = normal.dot(Vec3 { x: 0.0, y: 0.0, z: -1.0 });
    //     let component = (light * 255.0) as u8;

    //     if light > 0.0 {
    //         triangle(&t0, &t1, &t2, depths, tex0, tex1, tex2, &mut image, &tex, &Color(component, component, component));
    //     }
    // }

}

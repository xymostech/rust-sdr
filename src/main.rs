#![allow(dead_code)]
extern crate imagefmt;

mod vec;
mod image;
mod obj;

use std::cmp;
use vec::{Vec2, Vec3};
use image::*;
use obj::*;

fn diff(a: usize, b: usize) -> usize {
    if a < b {
        b - a
    } else {
        a - b
    }
}

fn _line(x0: usize, y0: usize, x1: usize, y1: usize, image: &mut Image, color: &Color, flip: bool) {
    if diff(x1, x0) < diff(y1, y0) {
        _line(y0, x0, y1, x1, image, color, true)
    } else if x1 < x0 {
        _line(x1, y1, x0, y0, image, color, flip)
    } else if x0 == x1 && y0 == y1{
        image.set_pixel(x0, y0, color);
    } else {
        let mut x = x0;
        let mut y = y0;

        let dx = x1 - x0;
        let dy = diff(y1, y0);

        let mut ydiff = dx;

        while x <= x1 {
            if flip {
                image.set_pixel(y, x, color);
            } else {
                image.set_pixel(x, y, color);
            }

            x += 1;
            ydiff += dy * 2;

            if ydiff > 2 * dx {
                if y0 < y1 {
                    y += 1;
                } else {
                    y -= 1;
                }
                ydiff -= dx * 2;
            }
        }
    }
}

fn line(x0: usize, y0: usize, x1: usize, y1: usize, image: &mut Image, color: &Color) {
    _line(x0, y0, x1, y1, image, color, false)
}

fn face_line(obj: &Obj, p1: &FacePoint, p2: &FacePoint, image: &mut Image, color: &Color) {
    let v1 = obj.vert(p1.vindex);
    let v2 = obj.vert(p2.vindex);

    let x0 = (v1.x + 1.0) * 400.0;
    let x1 = (v2.x + 1.0) * 400.0;
    let y0 = (v1.y + 1.0) * 400.0;
    let y1 = (v2.y + 1.0) * 400.0;

    line(x0 as usize, y0 as usize, x1 as usize, y1 as usize, image, color);
}

fn bounding_box(points: &[&Vec2<isize>]) -> (Vec2<isize>, Vec2<isize>) {
    let mut xmin = isize::max_value();
    let mut ymin = isize::max_value();
    let mut xmax = isize::min_value();
    let mut ymax = isize::min_value();

    for point in points {
        xmin = cmp::min(xmin, point.x);
        ymin = cmp::min(ymin, point.y);
        xmax = cmp::max(xmax, point.x);
        ymax = cmp::max(ymax, point.y);
    }

    (Vec2::<isize> { x: xmin, y: ymin }, Vec2::<isize> { x: xmax, y: ymax })
}

fn barycentric(x: isize, y: isize, t0: &Vec2<isize>, t1: &Vec2<isize>, t2: &Vec2<isize>) -> Vec3<f32> {
    let c = Vec3::cross(
        Vec3 { x: (t2.x - t0.x) as f32, y: (t1.x - t0.x) as f32, z: (t0.x - x) as f32 },
        Vec3 { x: (t2.y - t0.y) as f32, y: (t1.y - t0.y) as f32, z: (t0.y - y) as f32 }
    );

    Vec3 {
        x: 1.0 - (c.x + c.y) / c.z,
        y: c.y / c.z,
        z: c.x / c.z,
    }
}

fn tex_color(p: Vec2<f32>, texture: &Image) -> Color {
    texture.get_pixel((p.x * texture.width as f32) as usize, (p.y * texture.height as f32) as usize)
}

fn triangle(t0: &Vec2<isize>, t1: &Vec2<isize>, t2: &Vec2<isize>, depths: Vec3<f32>, tex0: Vec2<f32>, tex1: Vec2<f32>, tex2: Vec2<f32>, image: &mut Image, tex: &Image, light_color: &Color) {
    let (bbmin, bbmax) = bounding_box(&vec![t0, t1, t2]);
    let min = Vec2::<isize> { x: 0, y: 0 };
    let max = Vec2::<isize> { x: (image.width - 1) as isize, y: (image.height - 1) as isize };
    let (bbmin, bbmax) = (Vec2::clamp(&bbmin, &min, &max), Vec2::clamp(&bbmax, &min, &max));

    for x in bbmin.x as usize..bbmax.x as usize {
        for y in bbmin.y as usize..bbmax.y as usize {
            let params = barycentric(
                x as isize, y as isize,
                t0, t1, t2
            );

            let inside_triangle = params.x >= 0.0 && params.y >= 0.0 && params.z >= 0.0;

            if inside_triangle {
                let depth = depths.x * params.x + depths.y * params.y + depths.z * params.z;

                let tex_coords = Vec2 {
                    x: tex0.x * params.x + tex1.x * params.y + tex2.x * params.z,
                    y: tex0.y * params.x + tex1.y * params.y + tex2.y * params.z,
                };
                let color = tex_color(tex_coords, tex);
                image.set_pixel_with_depth(x, y, &color.multiply(light_color), (depth * 400.0) as isize);
            }
        }
    }
}

fn main() {
    let obj = Obj::from_file("head.obj").unwrap();
    let tex = Image::from("head_tex.tga");

    let mut image = Image::new(800, 800);

    for face in obj.faces.iter() {
        let fp0 = obj.vert(face.0.vindex);
        let fp1 = obj.vert(face.1.vindex);
        let fp2 = obj.vert(face.2.vindex);

        let t0 = Vec2::<isize> { x: ((fp0.x + 1.0) * 400.0) as isize, y: ((fp0.y + 1.0) * 400.0) as isize };
        let t1 = Vec2::<isize> { x: ((fp1.x + 1.0) * 400.0) as isize, y: ((fp1.y + 1.0) * 400.0) as isize };
        let t2 = Vec2::<isize> { x: ((fp2.x + 1.0) * 400.0) as isize, y: ((fp2.y + 1.0) * 400.0) as isize };

        let tex0 = obj.tex_vert(face.0.tindex);
        let tex1 = obj.tex_vert(face.1.tindex);
        let tex2 = obj.tex_vert(face.2.tindex);

        let depths = Vec3 {
            x: fp0.z,
            y: fp1.z,
            z: fp2.z,
        };

        let normal = face.normal(&obj);

        let light = normal.dot(Vec3 { x: 0.0, y: 0.0, z: -1.0 });
        let component = (light * 255.0) as u8;

        if light > 0.0 {
            triangle(&t0, &t1, &t2, depths, tex0, tex1, tex2, &mut image, &tex, &Color(component, component, component));
        }
    }

    image.write("out.tga").unwrap();
}

#![allow(dead_code)]
extern crate imagefmt;
use imagefmt::{ColFmt, ColType};
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::cmp;

#[derive(Clone, Copy)]
struct Vec2<T> {
    x: T,
    y: T,
}

impl <T: Copy + Ord> Vec2<T> {
    fn clamp(&self, min: &Vec2<T>, max: &Vec2<T>) -> Vec2<T> {
        Vec2::<T> {
            x: cmp::max(min.x, cmp::min(max.x, self.x)),
            y: cmp::max(min.y, cmp::min(max.y, self.y)),
        }
    }
}

#[derive(Clone, Copy)]
struct Vec3<T> {
    x: T,
    y: T,
    z: T,
}

impl <T: Copy + std::ops::Mul<T, Output = T> + std::ops::Sub<T, Output = T>> Vec3<T> {
    fn cross(&self, other: &Vec3<T>) -> Vec3<T> {
        Vec3::<T> {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

#[derive(Clone, Copy)]
struct FacePoint {
    vindex: usize,
    tindex: usize,
    nindex: usize,
}
struct Face(FacePoint, FacePoint, FacePoint);

struct Obj {
    verts: Vec<Vec3<f32>>,
    tex_verts: Vec<Vec3<f32>>,
    norm_verts: Vec<Vec3<f32>>,
    faces: Vec<Face>,
}

fn parse_point(line: &String) -> Vec3<f32> {
    let mut iter = line.split_whitespace();

    iter.next();
    let vec: Vec<f32> = iter.map(|x| x.parse().unwrap()).collect();

    Vec3::<f32> { x: vec[0], y: vec[1], z: vec[2] }
}

impl Obj {
    fn vert(&self, i: usize) -> &Vec3<f32> {
        &self.verts[i - 1]
    }

    fn from_file(filename: &str) -> Result<Obj, std::io::Error> {
        let f = try!(File::open(filename));

        let mut obj = Obj {
            verts: Vec::new(),
            tex_verts: Vec::new(),
            norm_verts: Vec::new(),
            faces: Vec::new(),
        };

        let file = BufReader::new(&f);
        for line in file.lines() {
            let line = try!(line);

            if line.starts_with("v") {
                obj.verts.push(parse_point(&line));
            } else if line.starts_with("vt") {
                obj.tex_verts.push(parse_point(&line));
            } else if line.starts_with("vn") {
                obj.norm_verts.push(parse_point(&line));
            } else if line.starts_with("f") {
                let mut iter = line.split_whitespace();

                iter.next();
                let vec: Vec<&str> = iter.collect();
                let mut face_vec: Vec<FacePoint> = Vec::new();

                for face in vec {
                    let vec: Vec<usize> = face.split("/").map(|x| x.parse().unwrap()).collect();

                    face_vec.push(FacePoint { vindex: vec[0], tindex: vec[1], nindex: vec[2] });
                }

                obj.faces.push(Face(face_vec[0], face_vec[1], face_vec[2]));
            }
        }

        Ok(obj)
    }
}

struct Color(u8, u8, u8);

const WHITE: Color = Color(255, 255, 255);
const BLUE: Color = Color(0, 0, 255);
const RED: Color = Color(255, 0, 0);
const GREEN: Color = Color(0, 255, 0);

struct Image {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

impl Image {
    fn set_pixel(&mut self, x: usize, y: usize, color: &Color) {
        if x < self.width && y < self.height {
            let index = ((self.height - y - 1) * self.width + x) * 3;

            self.data[index] = color.0;
            self.data[index+1] = color.1;
            self.data[index+2] = color.2;
        }
    }

    fn write(&self, filename: &str) -> imagefmt::Result<()> {
        imagefmt::write(
            filename,
            self.width,
            self.height,
            ColFmt::RGB,
            &self.data,
            ColType::Color)
    }

    fn new(width: usize, height: usize) -> Image {
        let mut data = Vec::with_capacity(width * height * 3);
        data.resize(width * height * 3, 0);

        Image {
            data: data,
            width: width,
            height: height,
        }
    }
}

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

fn inside_triangle(p: &Vec2<isize>, t0: &Vec2<isize>, t1: &Vec2<isize>, t2: &Vec2<isize>) -> bool {
    let c = Vec3::cross(
        &Vec3::<isize> { x: t2.x - t0.x, y: t1.x - t0.x, z: t0.x - p.x },
        &Vec3::<isize> { x: t2.y - t0.y, y: t1.y - t0.y, z: t0.y - p.y }
    );

    if c.z < 0 {
        c.z - c.x - c.y <= 0 &&
            c.y <= 0 &&
            c.x <= 0
    } else {
        c.z - c.x - c.y >= 0 &&
            c.y >= 0 &&
            c.x >= 0
    }
}

fn triangle(t0: &Vec2<isize>, t1: &Vec2<isize>, t2: &Vec2<isize>, image: &mut Image, color: &Color) {
    let (bbmin, bbmax) = bounding_box(&vec![t0, t1, t2]);
    let min = Vec2::<isize> { x: 0, y: 0 };
    let max = Vec2::<isize> { x: (image.width - 1) as isize, y: (image.height - 1) as isize };
    let (bbmin, bbmax) = (Vec2::clamp(&bbmin, &min, &max), Vec2::clamp(&bbmax, &min, &max));

    for x in bbmin.x as usize..bbmax.x as usize {
        for y in bbmin.y as usize..bbmax.y as usize {
            if inside_triangle(&Vec2::<isize> { x: x as isize, y: y as isize }, t0, t1, t2) {
                image.set_pixel(x, y, color);
            }
        }
    }
}

fn main() {
    let obj = Obj::from_file("head.obj").unwrap();

    let mut image = Image::new(800, 800);

    let colors = [&RED, &GREEN, &WHITE, &BLUE];
    let mut color = 0;

    for face in obj.faces.iter() {
        let fp0 = obj.vert(face.0.vindex);
        let fp1 = obj.vert(face.1.vindex);
        let fp2 = obj.vert(face.2.vindex);;

        let t0 = Vec2::<isize> { x: ((fp0.x + 1.0) * 400.0) as isize, y: ((fp0.y + 1.0) * 400.0) as isize };
        let t1 = Vec2::<isize> { x: ((fp1.x + 1.0) * 400.0) as isize, y: ((fp1.y + 1.0) * 400.0) as isize };
        let t2 = Vec2::<isize> { x: ((fp2.x + 1.0) * 400.0) as isize, y: ((fp2.y + 1.0) * 400.0) as isize };

        triangle(&t0, &t1, &t2, &mut image, &colors[color % 4]);
        color += 1;
    }

    image.write("out.tga").unwrap();
}

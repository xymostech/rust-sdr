#![allow(dead_code)]
extern crate imagefmt;
use imagefmt::{ColFmt, ColType};
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::cmp;
use std::ops::{Sub, Add, Mul};

#[derive(Clone, Copy)]
struct Vec2<T> {
    x: T,
    y: T,
}

impl<T: Copy + Ord> Vec2<T> {
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
    fn cross(self, other: Vec3<T>) -> Vec3<T> {
        Vec3::<T> {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

impl<T: Copy + Mul<T, Output = T> + Add<T, Output = T>> Vec3<T> {
    fn dot(self, other: Vec3<T>) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl Vec3<f32> {
    fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn norm(self) -> Vec3<f32> {
        let length = self.length();

        Vec3 {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
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

impl Face {
    fn normal(&self, obj: &Obj) -> Vec3<f32> {
        let v0 = obj.vert(self.0.vindex);
        let v1 = obj.vert(self.1.vindex);
        let v2 = obj.vert(self.2.vindex);

        (v2 - v0).cross(v1 - v0).norm()
    }
}

impl Obj {
    fn vert(&self, i: usize) -> Vec3<f32> {
        self.verts[i - 1]
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
    zbuffer: Vec<isize>,
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

    fn set_pixel_with_depth(&mut self, x: usize, y: usize, color: &Color, depth: isize) {
        if x < self.width && y < self.height {
            let zbufferindex = (self.height - y - 1) * self.width + x;
            let index = zbufferindex * 3;

            if self.zbuffer[zbufferindex] < depth {
                self.zbuffer[zbufferindex] = depth;

                self.data[index] = color.0;
                self.data[index+1] = color.1;
                self.data[index+2] = color.2;
            }
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
        let mut zbuffer = Vec::with_capacity(width * height);
        zbuffer.resize(width * height, isize::min_value());

        Image {
            data: data,
            zbuffer: zbuffer,
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

fn triangle(t0: &Vec2<isize>, t1: &Vec2<isize>, t2: &Vec2<isize>, depths: Vec3<f32>, image: &mut Image, color: &Color) {
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
                image.set_pixel_with_depth(x, y, color, (depth * 400.0) as isize);
            }
        }
    }
}

fn main() {
    let obj = Obj::from_file("head.obj").unwrap();

    let mut image = Image::new(800, 800);

    for face in obj.faces.iter() {
        let fp0 = obj.vert(face.0.vindex);
        let fp1 = obj.vert(face.1.vindex);
        let fp2 = obj.vert(face.2.vindex);

        let t0 = Vec2::<isize> { x: ((fp0.x + 1.0) * 400.0) as isize, y: ((fp0.y + 1.0) * 400.0) as isize };
        let t1 = Vec2::<isize> { x: ((fp1.x + 1.0) * 400.0) as isize, y: ((fp1.y + 1.0) * 400.0) as isize };
        let t2 = Vec2::<isize> { x: ((fp2.x + 1.0) * 400.0) as isize, y: ((fp2.y + 1.0) * 400.0) as isize };

        let depths = Vec3 {
            x: fp0.z,
            y: fp1.z,
            z: fp2.z,
        };

        let normal = face.normal(&obj);

        let light = normal.dot(Vec3 { x: 0.0, y: 0.0, z: -1.0 });
        let component = (light * 255.0) as u8;

        if light > 0.0 {
            triangle(&t0, &t1, &t2, depths, &mut image, &Color(component, component, component));
        }
    }

    image.write("out.tga").unwrap();
}

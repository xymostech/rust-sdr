#![allow(dead_code)]
extern crate imagefmt;
use imagefmt::{ColFmt, ColType};
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::cmp;

struct Point(f32, f32, f32);
#[derive(Clone)]
struct FacePoint(usize, usize, usize);
struct Face(FacePoint, FacePoint, FacePoint);

struct Obj {
    verts: Vec<Point>,
    tex_verts: Vec<Point>,
    norm_verts: Vec<Point>,
    faces: Vec<Face>,
}

fn parse_point(line: &String) -> Point {
    let mut iter = line.split_whitespace();

    iter.next();
    let vec: Vec<f32> = iter.map(|x| x.parse().unwrap()).collect();

    Point(vec[0], vec[1], vec[2])
}

impl Obj {
    fn vert(&self, i: usize) -> &Point {
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

                    face_vec.push(FacePoint(vec[0], vec[1], vec[2]));
                }

                obj.faces.push(Face(face_vec[0].clone(), face_vec[1].clone(), face_vec[2].clone()));
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
    let v1 = &obj.verts[p1.0 - 1];
    let v2 = &obj.verts[p2.0 - 1];

    let x0 = (v1.0 + 1.0) * 400.0;
    let x1 = (v2.0 + 1.0) * 400.0;
    let y0 = (v1.1 + 1.0) * 400.0;
    let y1 = (v2.1 + 1.0) * 400.0;

    line(x0 as usize, y0 as usize, x1 as usize, y1 as usize, image, color);
}

struct Vec2 {
    x: isize,
    y: isize,
}

struct Vec3 {
    x: isize,
    y: isize,
    z: isize,
}

fn bounding_box(points: &[&Vec2]) -> (Vec2, Vec2) {
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

    (Vec2 { x: xmin, y: ymin }, Vec2 { x: xmax, y: ymax })
}

fn clamp(v: &Vec2, min: &Vec2, max: &Vec2) -> Vec2 {
    Vec2 { x: cmp::max(min.x, cmp::min(max.x, v.x)), y: cmp::max(min.y, cmp::min(max.y, v.y)) }
}

fn cross(a: &Vec3, b: &Vec3) -> Vec3 {
    Vec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

fn inside_triangle(p: &Vec2, t0: &Vec2, t1: &Vec2, t2: &Vec2) -> bool {
    let c = cross(
        &Vec3 { x: t2.x - t0.x, y: t1.x - t0.x, z: t0.x - p.x },
        &Vec3 { x: t2.y - t0.y, y: t1.y - t0.y, z: t0.y - p.y }
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

fn triangle(t0: &Vec2, t1: &Vec2, t2: &Vec2, image: &mut Image, color: &Color) {
    let (bbmin, bbmax) = bounding_box(&vec![t0, t1, t2]);
    let min = Vec2 { x: 0, y: 0 };
    let max = Vec2 { x: (image.width - 1) as isize, y: (image.height - 1) as isize };
    let (bbmin, bbmax) = (clamp(&bbmin, &min, &max), clamp(&bbmax, &min, &max));

    for x in bbmin.x as usize..bbmax.x as usize {
        for y in bbmin.y as usize..bbmax.y as usize {
            if inside_triangle(&Vec2 { x: x as isize, y: y as isize }, t0, t1, t2) {
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
        let fp0 = obj.vert((face.0).0);
        let fp1 = obj.vert((face.1).0);
        let fp2 = obj.vert((face.2).0);

        let t0 = Vec2 { x: ((fp0.0 + 1.0) * 400.0) as isize, y: ((fp0.1 + 1.0) * 400.0) as isize };
        let t1 = Vec2 { x: ((fp1.0 + 1.0) * 400.0) as isize, y: ((fp1.1 + 1.0) * 400.0) as isize };
        let t2 = Vec2 { x: ((fp2.0 + 1.0) * 400.0) as isize, y: ((fp2.1 + 1.0) * 400.0) as isize };

        triangle(&t0, &t1, &t2, &mut image, &colors[color % 4]);
        color += 1;
    }

    image.write("out.tga").unwrap();
}

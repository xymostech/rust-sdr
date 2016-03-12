#![allow(dead_code)]
extern crate imagefmt;
use imagefmt::{ColFmt, ColType};
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

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
    x: usize,
    y: usize,
}

fn triangle(t0: &Vec2, t1: &Vec2, t2: &Vec2, image: &mut Image, color: &Color) {
    if t0.y > t1.y {
        triangle(t1, t0, t2, image, color);
    } else if t1.y > t2.y {
        triangle(t0, t2, t1, image, color);
    } else {
        //line(t0.x, t0.y, t1.x, t1.y, image, &WHITE);
        //line(t1.x, t1.y, t2.x, t2.y, image, &WHITE);
        //line(t2.x, t2.y, t0.x, t0.y, image, &WHITE);

        let total_height = t2.y - t0.y;
        let segment_height = t1.y - t0.y;

        let mut x1 = t0.x;
        let mut x2 = t0.x;

        let (dx1, x1neg, x1height, dx2, x2neg, x2height) = if t1.x < t2.x {
            (diff(t0.x, t1.x), t1.x < t0.x, segment_height, diff(t0.x, t2.x), t2.x < t0.x, total_height)
        } else {
            (diff(t0.x, t2.x), t2.x < t0.x, total_height, diff(t0.x, t1.x), t1.x < t0.x, segment_height)
        };

        let mut x1diff = x1height;
        let mut x2diff = x2height;

        for y in t0.y..(t1.y + 1) {
            for x in x1..(x2+1) {
                image.set_pixel(x, y, color);
            }

            x1diff += dx1 * 2;
            x2diff += dx2 * 2;

            while x1diff > 2 * x1height {
                if x1neg {
                    x1 -= 1;
                } else {
                    x1 += 1;
                }
                x1diff -= 2 * x1height;
            }
            while x2diff > 2 * x2height {
                if x2neg {
                    x2 -= 1;
                } else {
                    x2 += 1;
                }
                x2diff -= 2 * x2height;
            }
        }

        let segment_height = t2.y - t1.y;

        let mut x1 = t2.x;
        let mut x2 = t2.x;

        let (dx1, x1neg, x1height, dx2, x2neg, x2height) = if t0.x < t1.x {
            (diff(t0.x, t2.x), t0.x < t2.x, total_height, diff(t1.x, t2.x), t1.x < t2.x, segment_height)
        } else {
            (diff(t1.x, t2.x), t1.x < t2.x, segment_height, diff(t0.x, t2.x), t0.x < t2.x, total_height)
        };

        let mut x1diff = x1height;
        let mut x2diff = x2height;

        // t2.y down to t1.y
        for y in (t1.y..(t2.y + 1)).rev() {
            for x in x1..(x2+1) {
                image.set_pixel(x, y, color);
            }

            x1diff += dx1 * 2;
            x2diff += dx2 * 2;

            while x1diff > 2 * x1height {
                if x1neg {
                    x1 -= 1;
                } else {
                    x1 += 1;
                }
                x1diff -= 2 * x1height;
            }
            while x2diff > 2 * x2height {
                if x2neg {
                    x2 -= 1;
                } else {
                    x2 += 1;
                }
                x2diff -= 2 * x2height;
            }
        }
    }
}

fn main() {
    // let obj = Obj::from_file("head.obj").unwrap();

    let mut image = Image::new(200, 200);

    let t0: Vec<Vec2> = vec![Vec2 { x: 10, y: 70 },   Vec2 { x: 50, y: 16 },  Vec2 { x: 70, y: 80 }];
    let t1: Vec<Vec2> = vec![Vec2 { x: 180, y: 50 },  Vec2 { x: 150, y: 1 },   Vec2 { x: 70, y: 180 }];
    let t2: Vec<Vec2> = vec![Vec2 { x: 180, y: 150 }, Vec2 { x: 120, y: 160 }, Vec2 { x: 130, y: 180 }];

    triangle(&t0[0], &t0[1], &t0[2], &mut image, &RED);
    triangle(&t1[0], &t1[1], &t1[2], &mut image, &WHITE);
    triangle(&t2[0], &t2[1], &t2[2], &mut image, &GREEN);

    image.write("out.tga").unwrap();
}

use vec::{Vec2, Vec3, Vec4};
use image::{Image, Color};
use std::cmp;

pub trait Shader {
    fn vertex(&self, Vec3<f32>) -> (Vec4<f32>, Vec3<f32>);
    fn fragment(&self, Vec2<isize>, Vec3<f32>) -> Option<Color>;
}

fn barycentric(point: Vec2<isize>, verts: &Vec<Vec2<isize>>) -> Vec3<f32> {
    let c = Vec3::cross(
        Vec3 { x: (verts[2].x - verts[0].x) as f32, y: (verts[1].x - verts[0].x) as f32, z: (verts[0].x - point.x) as f32 },
        Vec3 { x: (verts[2].y - verts[0].y) as f32, y: (verts[1].y - verts[0].y) as f32, z: (verts[0].y - point.y) as f32 },
    );

    Vec3 {
        x: 1.0 - (c.x + c.y) / c.z,
        y: c.y / c.z,
        z: c.x / c.z,
    }
}

fn bounding_box<T: cmp::Ord + Copy>(pts: &Vec<Vec2<T>>) -> (Vec2<T>, Vec2<T>) {
    let mut min: Vec2<T> = pts[0];
    let mut max: Vec2<T> = pts[0];

    for pt in pts {
        min.x = cmp::min(min.x, pt.x);
        min.y = cmp::min(min.y, pt.y);

        max.x = cmp::max(max.x, pt.x);
        max.y = cmp::max(max.y, pt.y);
    }

    (min, max)
}

pub fn draw_triangle<S: Shader>(verts: &Vec<Vec3<f32>>, shader: &S, image: &mut Image) {
    let vertex_outs: Vec<(Vec4<f32>, Vec3<f32>)> = verts.iter().map(|v| shader.vertex(*v)).collect();
    let depths: Vec<f32> = vertex_outs.iter().map(|&(v, _)| v.z).collect();
    let xy_verts: Vec<Vec2<isize>> =
        vertex_outs.iter()
        .map(|&(v, _)| v.xy() / v.w)
        .map(|v| Vec2 { x: v.x as isize, y: v.y as isize })
        .collect();
    let colors: Vec<Vec3<f32>> = vertex_outs.iter().map(|&(_, c)| c).collect();

    let (min_bb, max_bb) = bounding_box(&xy_verts);

    for x in cmp::max(0, min_bb.x)..cmp::min((image.width - 1) as isize, max_bb.x) {
        for y in cmp::max(0, min_bb.y)..cmp::min((image.height - 1) as isize, max_bb.y) {
            let pt = Vec2 { x: x, y: y };

            let bary = barycentric(pt, &xy_verts);

            if bary.x < 0.0 || bary.y < 0.0 || bary.z < 0.0 {
                continue;
            }

            let in_color = colors[0] * bary.x + colors[1] * bary.y + colors[2] * bary.z;
            let maybe_out_color = shader.fragment(pt, in_color);

            match maybe_out_color {
                Some(out_color) => {
                    let depth = depths[0] * bary.x + depths[1] * bary.y + depths[2] * bary.z;
                    image.set_pixel_with_depth(x as usize, y as usize, &out_color, depth as isize);
                }
                None => {}
            }

        }
    }
}

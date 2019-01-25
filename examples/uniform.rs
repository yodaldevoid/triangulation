use std::path::PathBuf;

use structopt::StructOpt;
use image::RgbImage;
use imageproc::drawing::{draw_antialiased_line_segment_mut, draw_filled_rect_mut};
use imageproc::rect::Rect;
use rand::Rng;

use triangulation::{Delaunay, Point};

/// Triangulates a set of normally distributed random points
/// and saves the result as a PNG image
#[derive(StructOpt, Debug)]
#[structopt(name = "uniform")]
struct Opt {
    /// Number of points
    #[structopt(short = "c", long = "count", default_value = "1000")]
    count: usize,

    /// Output image height
    #[structopt(short = "w", long = "width", default_value = "1000")]
    width: u32,

    /// Output image width
    #[structopt(short = "h", long = "height", default_value = "1000")]
    height: u32,

    /// Output file
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    let mut rng = rand::thread_rng();
    let mut points = vec![];

    for _ in 0..opt.count {
        let x = rng.gen_range(0.0, opt.width as f32);
        let y = rng.gen_range(0.0, opt.height as f32);
        points.push(Point::new(x, y));
    }

    let t = std::time::Instant::now();
    let triangulation = Delaunay::new(&points).unwrap();
    println!("Created {} triangles in {:?}", triangulation.triangles.len(), t.elapsed());

    let t = std::time::Instant::now();
    let mut im = image::DynamicImage::new_rgb8(opt.width, opt.height);
    let im = im.as_mut_rgb8().unwrap();

    draw_filled_rect_mut(
        im,
        Rect::at(0, 0).of_size(opt.width, opt.height),
        image::Rgb([255, 255, 255]),
    );

    fn draw_line(im: &mut RgbImage, a: Point, b: Point) {
        draw_antialiased_line_segment_mut(
            im,
            a.into(),
            b.into(),
            image::Rgb([0, 0, 0]),
            |new, old, fac| {
                let r = f32::from(new.data[0]) * fac + f32::from(old.data[0]) * (1.0 - fac);
                let g = f32::from(new.data[1]) * fac + f32::from(old.data[1]) * (1.0 - fac);
                let b = f32::from(new.data[2]) * fac + f32::from(old.data[2]) * (1.0 - fac);
                image::Rgb([r as u8, g as u8, b as u8])
            },
        );
    };

    for i in (0..triangulation.triangles.len()).step_by(3) {
        let a = points[triangulation.triangles[i]];
        let b = points[triangulation.triangles[i + 1]];
        let c = points[triangulation.triangles[i + 2]];

        draw_line(im, a, b);
        draw_line(im, b, c);
        draw_line(im, c, a);
    }

    println!("Drawing took {:?}", t.elapsed());

    let t = std::time::Instant::now();
    im.save(&opt.output).unwrap();

    println!("Saved as {} in {:?}", opt.output.display(), t.elapsed());
}

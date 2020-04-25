use contour::ContourBuilder;
use palette::Srgb;
use palette::{
    named::{BLACK, WHITE},
    Gradient,
};
use plotters::prelude::*;
use ucsf_nmr::UcsfFile;

fn single_contour(polygon: &Vec<Vec<Vec<f64>>>) -> Vec<(i32, i32)> {
    polygon[0]
        .clone()
        .into_iter()
        .map(|n| ((n[0] * 2f64) as i32, (n[1] * 2f64) as i32))
        .collect()
}

fn draw_contour<DB: DrawingBackend>(
    drawing_area: &DrawingArea<DB, plotters::coord::RangedCoord<RangedCoordi32, RangedCoordi32>>,
    contour_builder: &ContourBuilder,
    data: &[f64],
    threshold: f64,
    shape_style: ShapeStyle,
) {
    // let drawing_area = drawing_area.into_drawing_area();
    let contours = contour_builder.contours(data, &[threshold]);

    let contour_group: Vec<geojson::PolygonType> =
        (match contours[0].geometry.clone().unwrap().value {
            geojson::Value::MultiPolygon(inner) => Some(inner.clone()),
            _ => None,
        })
        .unwrap();
    for contour in contour_group.iter() {
        let single_contour = single_contour(contour);

        let path = PathElement::new(single_contour, shape_style.clone());

        drawing_area.draw(&path).unwrap();
    }
}

pub fn main() {
    let contents = include_bytes!("../data/15n_hsqc.ucsf");
    // let contents = include_bytes!("../data/Nhsqc_highres_600MHz.ucsf");

    let (_, contents) = UcsfFile::parse(&contents[..]).expect("Failed parsing");
    let data: Vec<_> = contents
        .data_continous()
        .into_iter()
        .map(|n| n as f64)
        .collect();

    let root = BitMapBackend::new(
        "output/contour.png",
        (
            contents.axis_data_points(1) * 2 as u32,
            contents.axis_data_points(0) * 2 as u32,
        ),
    )
    .into_drawing_area()
    .apply_coord_spec(RangedCoord::new(
        0..(contents.axis_data_points(1) * 2) as i32,
        0..(contents.axis_data_points(0) * 2) as i32,
        (
            0..(contents.axis_data_points(1) * 2) as i32,
            (contents.axis_data_points(0) * 2) as i32..0,
        ),
    ));

    let (min_val, max_val) = contents.bounds();
    let contour_builder = ContourBuilder::new(
        contents.axis_data_points(1) as u32,
        contents.axis_data_points(0) as u32,
        true,
    );

    let gradient = Gradient::with_domain(vec![
        (min_val, Srgb::<f32>::from_format(WHITE).into_linear()),
        (max_val, Srgb::<f32>::from_format(BLACK).into_linear()),
    ]);

    let num_steps = 100;
    let step_size = (max_val - min_val) / num_steps as f32;
    let mut threshold = min_val;
    for _ in 0..num_steps {
        let shape_style = gradient.get(threshold).to_rgba().stroke_width(1);
        draw_contour(
            &root,
            &contour_builder,
            &data,
            threshold as f64,
            shape_style,
        );
        threshold += step_size;
    }
}

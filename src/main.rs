use eframe::egui;
use nalgebra::constraint;
use egui::epaint::{self, CubicBezierShape, PathShape, QuadraticBezierShape};
use std::collections::BTreeMap;
type Point3 = nalgebra::Point3<f32>;
//TODO: boolean, epxlicit parameters

// A fully specified shape we pass to manifoldcad, ready for rendering
// TODO(Dhruv) include IDs here so troy can send them back when users force points in the ui
enum DeterminedShape {
    Point(Point3),
    Line(Point3, Point3),
    Circle {
        radius: f32,
        origin: Point3
    }
}

// Give everyone IDs (TODO)

// A relation that is passed to the solver
struct Relation;

// An expression that a user types in
//struct Expr;
//

// stopgap until we add user-defined expression support
type Expr = f32;
type Parameter = f32;


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PointId(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CircleId(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SegmentId(pub usize);

pub struct Point {
    pub x: Parameter,
    pub y: Parameter,
    pub z: Parameter,
}

pub struct Segment {
    pub a: PointId,
    pub b: PointId
}

pub struct Circle {
    pub radius: Parameter,
    pub origin: PointId
}

enum Constraint {
    PointOnCircle(PointId, CircleId),
    PointOnLine(PointId, SegmentId),
    Distance(PointId, PointId, Expr),
}

#[derive(Default)]
pub struct Objects {
    pub points: BTreeMap<PointId, Point>,
    pub segments: BTreeMap<SegmentId, Segment>,
    pub circles: BTreeMap<CircleId, Circle>,
}

/*fn make_system(constraints: Vec<Constraint>, objects: Objects) ->  {
}*/

/// Solves the Objects and Constraints, substitutes in initial conditions where necessary, and
/// returns a determined system
fn make_determinate(objects: Objects, constraints: Vec<Constraint>) -> Vec<DeterminedShape> {
    let mut determined = Vec::new();
    for point in objects.points.values() {
        determined.push(DeterminedShape::Point(Point3::new(point.x, point.y, point.z)));
    }

    determined
}

#[derive(Default)]
struct MyApp {
    name: String,
    age: u32,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
        });
    }
}


fn main() {
    let mut objects = Objects::default();
    let constraints = vec![];

    objects.points.insert(PointId(0), Point {
        x: 0.,
        y: 0.,
        z: 0.,
    });

    let determined_shapes = make_determinate(objects, constraints);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            Ok(Box::<MyApp>::default())
        }),
    ).expect("failed to start egui");
}

use eframe::egui;
use nalgebra::{constraint, Point3};
use egui::{epaint::{self, CubicBezierShape, PathShape, QuadraticBezierShape}, Color32, Pos2, Sense, Stroke, StrokeKind, Vec2};
use solve::{Circle, CircleId, Constraint, DeterminedShape, Objects, PointId, Segment, SegmentId};
mod solve;

/*fn make_system(constraints: Vec<Constraint>, objects: Objects) ->  {
}*/

//TODO: mark stuff as dirty

/// Solves the Objects and Constraints, substitutes in initial conditions where necessary, and
/// returns a determined system
fn make_determinate(objects: Objects, constraints: Vec<Constraint>) -> Vec<DeterminedShape> {
    let mut determined = Vec::new();

    for segment in objects.segments.values() {
        let a = objects.points.get(&segment.a).expect("TODO");
        let b = objects.points.get(&segment.b).expect("TODO");
        determined.push(DeterminedShape::Line(Point3::new(a.x, a.y, a.z), Point3::new(b.x, b.y, b.z)));
    }
    
    for circle in objects.circles.values() {
        let origin = objects.points.get(&circle.origin).expect("TODO");
        determined.push(DeterminedShape::Circle {
            radius: circle.radius,
            origin: Point3::new(origin.x, origin.y, origin.z)

        });
    }

    for point in objects.points.values() {
        determined.push(DeterminedShape::Point(Point3::new(point.x, point.y, point.z)));
    }

    determined
}

#[derive(Default)]
struct MyApp {
    shapes: Vec<DeterminedShape>
}

// boolean operations, extrusions, spheres, filet/chamfer extuded edges

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            let (response, painter) = ui.allocate_painter(egui::Vec2 { x: ui.available_width(), y: ui.available_height() }, Sense::hover());
            let rect = response.rect;
            let o = rect.center();

            let stroke = Stroke::new(2., Color32::WHITE);

            //painter.circle(rect.center(), rect.height()/2. - 2., Color32::RED, Stroke::default());
            for shape in &self.shapes {
                match shape {
                    DeterminedShape::Point(pt) => {
                        painter.rect(egui::Rect {
                            min: o + Vec2::new(pt.x-3., pt.y-3.),
                            max: o + Vec2::new(pt.x+3., pt.y+3.)
                        }, 0, Color32::GREEN, Stroke::default(), StrokeKind::Inside);
                    },
                    DeterminedShape::Line(pt1, pt2) => {
                        painter.line_segment([o + Vec2::new(pt1.x, pt1.y), o + Vec2::new(pt2.x, pt2.y)], stroke.clone());
                    },
                    DeterminedShape::Circle { radius, origin } => {
                        painter.circle_stroke(o + Vec2 { x: origin.x, y: origin.y }, *radius, stroke.clone());
                    },
                }
            };
        });
    }
}


fn main() {
    let mut objects = Objects::default();
    let constraints = vec![];

    objects.points.insert(PointId(0), solve::Point {
        x: 0.,
        y: 0.,
        z: 0.,
    });

    objects.points.insert(PointId(1), solve::Point {
        x: 50.,
        y: 50.,
        z: 0.,
    });

    objects.points.insert(PointId(2), solve::Point {
        x: 50.,
        y: -50.,
        z: 0.,
    });

    objects.points.insert(PointId(3), solve::Point {
        x: -50.,
        y: -50.,
        z: 0.,
    });



    objects.segments.insert(SegmentId(0), Segment {
        a: PointId(0),
        b: PointId(1),
    });

    objects.segments.insert(SegmentId(1), Segment {
        a: PointId(1),
        b: PointId(2),
    });

    objects.segments.insert(SegmentId(2), Segment {
        a: PointId(2),
        b: PointId(0),
    });

    objects.circles.insert(CircleId(0), Circle {
        radius: 30.,
        origin: PointId(3),
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
            Ok(Box::new(
                    MyApp {
                        shapes: determined_shapes
                    }
                    ))
        }),
    ).expect("failed to start egui");
}

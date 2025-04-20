use eframe::egui;
use nalgebra::{constraint, Point3};
use egui::{epaint::{self, CubicBezierShape, PathShape, QuadraticBezierShape}, Color32, Pos2, Sense, Stroke, StrokeKind, Vec2};
use solve::{Constraint, DeterminedShape, Objects, PointId};
mod solve;

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
    shapes: Vec<DeterminedShape>
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            let (response, painter) = ui.allocate_painter(egui::Vec2 { x: ui.available_width(), y: ui.available_height() }, Sense::hover());
            let rect = response.rect;
            let o = rect.center();

            //painter.circle(rect.center(), rect.height()/2. - 2., Color32::RED, Stroke::default());
            for shape in &self.shapes {
                match shape {
                    DeterminedShape::Point(pt) => {
                        painter.rect(egui::Rect {
                            min: o + Vec2::new(pt.x-2., pt.y-2.),
                            max: o + Vec2::new(pt.x+2., pt.y+2.)
                        }, 0, Color32::DARK_GREEN, Stroke::default(), StrokeKind::Inside);
                    },
                    DeterminedShape::Line(opoint, opoint1) => (),
                    DeterminedShape::Circle { radius, origin } => (),
                }
            }
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

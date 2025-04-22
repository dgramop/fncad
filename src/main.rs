use eframe::egui;
use nalgebra::{constraint, Point3};
use egui::{epaint::{self, CubicBezierShape, PathShape, QuadraticBezierShape}, Color32, Pos2, Sense, Stroke, StrokeKind, Vec2};
use solve::{Circle, CircleId, Constraint, DeterminedShape, Objects, Parameter, PointId, Segment, SegmentId};
mod solve;

/*fn make_system(constraints: Vec<Constraint>, objects: Objects) ->  {
}*/

struct MyApp {
    shapes: Objects
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
            for pt in &self.shapes.points {
                painter.rect(egui::Rect {
                    min: o + Vec2::new(self.shapes.get_parameter(pt.x).unwrap().value-3., self.shapes.get_parameter(pt.y).unwrap().value-3.),
                    max: o + Vec2::new(self.shapes.get_parameter(pt.x).unwrap().value+3., self.shapes.get_parameter(pt.y).unwrap().value+3.)
                }, 0, Color32::GREEN, Stroke::default(), StrokeKind::Inside);
            };

            for Segment { a, b } in &self.shapes.segments {
                let pt1 = self.shapes.get_point(*a).expect("Point used by segment exists");
                let pt2 = self.shapes.get_point(*b).expect("Point used by segment exists");
                painter.line_segment([o + Vec2::new(self.shapes.get_parameter(pt1.x).unwrap().value, self.shapes.get_parameter(pt1.y).unwrap().value), o + Vec2::new(self.shapes.get_parameter(pt2.x).unwrap().value, self.shapes.get_parameter(pt2.y).unwrap().value)], stroke.clone());
            }

            for Circle { radius, origin } in &self.shapes.circles {
                let origin = self.shapes.get_point(*origin).expect("Point used by circle exists");
                painter.circle_stroke(o + Vec2 { x: self.shapes.get_parameter(origin.x).unwrap().value, y: self.shapes.get_parameter(origin.y).unwrap().value }, self.shapes.get_parameter(*radius).unwrap().value, stroke.clone());
            }

        });
    }
}

fn main() {
    let mut objects = Objects::default();

    let origin = objects.add_point(Parameter::fixed(0.), Parameter::fixed(0.), Parameter::fixed(0.));

    let point = objects.add_point(Parameter::fixed(-50.), Parameter::fixed(0.), Parameter::fixed(0.));
    
    let circle = objects.add_circle(origin, Parameter::free(25.));

    let constraints = vec![Constraint::PointOnCircle(point, circle)];

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            Ok(
                Box::new(
                    MyApp {
                        shapes: objects
            }))
        }),
    ).expect("failed to start egui");
}

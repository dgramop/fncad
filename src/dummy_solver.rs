use anyhow::{Context, Result};
use log::info;
use std::thread::JoinHandle;

use crate::solve::{DeterminedShape, Point3, RendererInput};

pub enum SolverInput {
    TempAnimate { time_seconds: f32 },
}

#[derive(derive_new::new)]
pub struct Solver {
    renderer_tx: crossbeam_channel::Sender<RendererInput>,
    solver_rx: crossbeam_channel::Receiver<SolverInput>,
}

impl Solver {
    /// Spawns the solver thread, which will process updates and send new geometry until
    /// `solver_rx` is dropped.
    pub fn run(self) -> JoinHandle<Result<()>> {
        std::thread::spawn(move || {
            // NOTE: Have vertex a pushed further into the screen to test 3D line rotation
            let vertex_a = Point3::new(0.0, -1.0, -2.0);
            let vertex_b = Point3::new(-1.0, 1.0, -1.0);
            let vertex_c = Point3::new(1.0, 1.0, -1.0);

            let shapes = [
                DeterminedShape::Point(Point3::new(0.5, 0.5, 0.0)),
                DeterminedShape::Circle {
                    radius: 0.5,
                    origin: Point3::new(-3.0, 3.0, -3.0),
                },
                DeterminedShape::Line(vertex_a, vertex_b),
                DeterminedShape::Line(vertex_b, vertex_c),
                DeterminedShape::Line(vertex_c, vertex_a),
            ];

            for (id, shape) in shapes.into_iter().enumerate() {
                self.renderer_tx
                    .send(RendererInput::SingleShape { id, shape })
                    .context("Failed to send shape")?;
            }

            loop {
                let Ok(input) = self.solver_rx.recv() else {
                    info!("Solver input channel dropped, exiting");
                    break;
                };

                // Example code:
                {
                    let SolverInput::TempAnimate { time_seconds } = &input;
                    let x = 2.0 * time_seconds.sin();
                    let y = 2.0 * time_seconds.cos();
                    self.renderer_tx
                        .send(RendererInput::SingleShape {
                            id: 0,
                            shape: DeterminedShape::Point(Point3::new(x, y, 0.0)),
                        })
                        .context("Failed to send shape")?;
                }

                // TODO: add variants to describe changes in shape, use to re-solve stuff
                let _ = input;
            }
            //
            Ok(())
        })
    }
}

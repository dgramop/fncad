use crate::{
    dummy_solver::SolverInput,
    solve::{DeterminedShape, RendererInput},
};
use anyhow::{Result, anyhow};
use log::info;
use std::{collections::HashMap, thread::JoinHandle, time::Instant};
use three_d::*;

pub struct Renderer {
    camera: Camera,
    control: OrbitControl,
    light: DirectionalLight,
    shapes: HashMap<usize, Gm<Mesh, PhysicalMaterial>>,
    solver_task: Option<JoinHandle<Result<()>>>,
    renderer_rx: crossbeam_channel::Receiver<RendererInput>,
    solver_tx: crossbeam_channel::Sender<SolverInput>,
    axes: Axes,
    point_size: f32,
    line_size: f32,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let context = window.gl();

        let camera = Camera::new_orthographic(
            window.viewport(),
            vec3(0.0, 0.0, 5.0),
            vec3(0.0, 0.0, -1.0),
            vec3(0.0, 1.0, 0.0),
            1.0,
            0.001,
            1000.0,
        );
        let control = OrbitControl::new(camera.target(), 1.0, 100.0);

        let light = DirectionalLight::new(&context, 1.0, Srgba::WHITE, vec3(0.0, -0.5, -0.5));

        let axes = Axes::new(&context, 0.025, 1.0);

        let (renderer_tx, renderer_rx) = crossbeam_channel::bounded(64);
        let (solver_tx, solver_rx) = crossbeam_channel::bounded(64);

        let solver = crate::dummy_solver::Solver::new(renderer_tx, solver_rx);
        let solver_task = solver.run();

        Self {
            camera,
            control,
            light,
            shapes: HashMap::new(),
            solver_task: Some(solver_task),
            renderer_rx,
            solver_tx,
            axes,
            point_size: 0.1,
            line_size: 0.25,
        }
    }

    pub fn run(mut self, window: Window) -> Result<()> {
        let context = window.gl();
        let mut gui = three_d::GUI::new(&context);
        let solver_task = self.solver_task.take().unwrap();

        let mut last_render_time_ms = 0.0;
        window.render_loop(move |mut frame_input| {
            let start = Instant::now();
            let time_seconds = (frame_input.accumulated_time / 1000.0) as f32;
            self.update_solver(time_seconds);

            let mut panel_width = 0.0;
            gui.update(
                &mut frame_input.events,
                frame_input.accumulated_time,
                frame_input.viewport,
                frame_input.device_pixel_ratio,
                |gui_context| {
                    self.render_gui(
                        gui_context,
                        frame_input.elapsed_time / 1000.0,
                        last_render_time_ms,
                    );
                    panel_width = gui_context.used_rect().width();
                },
            );
            let viewport = Viewport {
                x: (panel_width * frame_input.device_pixel_ratio) as i32,
                y: 0,
                width: frame_input.viewport.width
                    - (panel_width * frame_input.device_pixel_ratio) as u32,
                height: frame_input.viewport.height,
            };
            self.camera.set_viewport(viewport);

            self.control
                .handle_events(&mut self.camera, &mut frame_input.events);

            self.update_shapes(&context);

            frame_input
                .screen()
                .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
                .render(&self.camera, self.render(), &[&self.light])
                .write(|| gui.render())
                .unwrap();

            last_render_time_ms = start.elapsed().as_secs_f64() * 1000.0;
            FrameOutput::default()
        });
        // TODO: Somehow get stop event and cleanly shutdown solver.
        // This code never runs, three_d simply calls exit() inside
        solver_task
            .join()
            .map_err(|e| anyhow!("Failed to join solver thread: {e:?}"))??;
        Ok(())
    }

    fn render(&self) -> impl Iterator<Item = impl three_d::Object> {
        self.axes
            .into_iter()
            .chain(self.shapes.iter().map(|s| s.1.into_iter()).flatten())
    }

    fn update_shapes(&mut self, context: &Context) {
        for input in self.renderer_rx.try_iter() {
            match input {
                RendererInput::SingleShape { id, shape } => {
                    let material = PhysicalMaterial::new(
                        context,
                        &CpuMaterial {
                            albedo: Srgba {
                                r: 128,
                                g: 128,
                                b: 128,
                                a: 255,
                            },
                            ..Default::default()
                        },
                    );
                    let mesh = match shape {
                        DeterminedShape::Point(point) => {
                            info!("Got point at {point:?}");
                            let mut mesh = Mesh::new(context, &CpuMesh::square());

                            let translate_mat =
                                Mat4::from_translation(vec3(point.x, point.y, point.z));
                            let scale_mat = Mat4::from_scale(self.point_size);
                            let transform = translate_mat * scale_mat;

                            mesh.set_transformation(transform);
                            mesh
                        }
                        DeterminedShape::Line(a, b) => {
                            let a = vec3(a.x, a.y, a.z);
                            let b = vec3(b.x, b.y, b.z);

                            let mut mesh = Mesh::new(context, &create_line(self.line_size));
                            let line_length = (a - b).magnitude();
                            let to_b = b - a;
                            let rotation = Quaternion::from_arc(vec3(1.0, 0.0, 0.0), to_b, None);

                            let translate_mat = Mat4::from_translation(a);
                            let scale_mat = Mat4::from_nonuniform_scale(line_length, 1.0, 1.0);
                            let transform = translate_mat * Mat4::from(rotation) * scale_mat;

                            mesh.set_transformation(transform);
                            mesh
                        }
                        DeterminedShape::Circle { radius, origin } => {
                            let mut mesh = Mesh::new(context, &CpuMesh::circle(100));
                            let transform =
                                Mat4::from_translation(vec3(origin.x, origin.y, origin.z))
                                    * Mat4::from_scale(radius);
                            mesh.set_transformation(transform);
                            mesh
                        }
                    };
                    self.shapes.insert(id, Gm::new(mesh, material));
                }
            }
        }
    }

    fn render_gui(&mut self, gui_context: &egui::Context, dt: f64, last_render_time_ms: f64) {
        use three_d::egui::*;
        SidePanel::left("side_panel").show(gui_context, |ui| {
            use three_d::egui::*;
            ui.heading("Debug Panel");
            ui.add(Label::new(format!("Render time: {:.1}ms", last_render_time_ms)));
            ui.add(Label::new(format!("Delta frame time: {:.1}ms", dt * 1000.0)));
            ui.add(Label::new(format!("FPS: {:.1}", 1.0 / dt)));

            ui.add(Slider::new(&mut self.point_size, 0.0001..=1.0).text("Point size"));
            // NOTE: Doenst do anything, since we only read from line_size on startup when
            // receiving initial shapes from the solver
            // ui.add(Slider::new(&mut self.line_size, 0.0001..=1.0).text("Line size"));
        });
    }

    fn update_solver(&self, time_seconds: f32) {
        let _ = self
            .solver_tx
            .try_send(SolverInput::TempAnimate { time_seconds });
    }
}

// Creates a line between (0, 0, 0) and (1, 0, 0) with the given height
fn create_line(line_size: f32) -> CpuMesh {
    let indices = vec![0u8, 1, 2, 2, 3, 0];

    let halfheight = line_size / 2.0;
    let positions = vec![
        Vec3::new(0.0, -halfheight, 0.0),
        Vec3::new(1.0, -halfheight, 0.0),
        Vec3::new(1.0, halfheight, 0.0),
        Vec3::new(0.0, halfheight, 0.0),
    ];
    let normals = vec![
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
    ];
    let tangents = vec![
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
    ];
    let uvs = vec![
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 0.0),
    ];
    CpuMesh {
        indices: Indices::U8(indices),
        positions: Positions::F32(positions),
        normals: Some(normals),
        tangents: Some(tangents),
        uvs: Some(uvs),
        ..Default::default()
    }
}

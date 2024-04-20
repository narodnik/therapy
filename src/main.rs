#![feature(stmt_expr_attributes)]
use glam::Vec4Swizzles;
use miniquad::*;
use std::{
    collections::HashMap,
    sync::{mpsc, Arc, Mutex},
    time,
};

#[macro_use]
extern crate log;
use log::LevelFilter;

type BoxResult = Result<(), Box<dyn std::error::Error>>;

enum CommandRequest {
    Hello,
    DrawLine(String, f32, f32, f32, f32, f32, f32, f32, f32, f32),
    Pan(f32, f32),
    Zoom(f32),
    ScreenToWorld(f32, f32),
    GetLayers,
    DeleteLayer(String),
    ShowLayer(String),
    HideLayer(String),
    SetLayerPos(String, f32, f32),
    ScreenSize,
}

enum CommandResponse {
    Hello(String),
    ScreenToWorld(f32, f32),
    GetLayers(Vec<String>),
    DeleteLayer(bool),
    ShowLayer(bool),
    HideLayer(bool),
    SetLayerPos(bool),
    ScreenSize(f32, f32),
}

trait MouseButtonAsString {
    fn to_str(&self) -> &str;
}

impl MouseButtonAsString for MouseButton {
    fn to_str(&self) -> &str {
        match self {
            MouseButton::Right => "Right",
            MouseButton::Left => "Left",
            MouseButton::Middle => "Middle",
            MouseButton::Unknown => "Unknown",
        }
    }
}

#[repr(C)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 4],
    uv: [f32; 2],
}

struct Layer {
    model: glam::Mat4,
    verts: Vec<Vertex>,
    idxs: Vec<u32>,
    is_hidden: bool,
}

impl Layer {
    fn new() -> Self {
        Self {
            model: glam::Mat4::IDENTITY,
            verts: vec![],
            idxs: vec![],
            is_hidden: false,
        }
    }
}

struct Stage {
    ctx: Box<dyn RenderingBackend>,
    pipeline: Pipeline,
    white_texture: TextureId,
    layers: HashMap<String, Layer>,
    proj: glam::Mat4,
    recv_req: mpsc::Receiver<CommandRequest>,
    send_res: mpsc::Sender<CommandResponse>,
    iface_ref: zbus::InterfaceRef<DbusApi>,
}

impl Stage {
    pub fn new(
        recv_req: mpsc::Receiver<CommandRequest>,
        send_res: mpsc::Sender<CommandResponse>,
        iface_ref: zbus::InterfaceRef<DbusApi>,
    ) -> Stage {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        let white_texture = ctx.new_texture_from_rgba8(1, 1, &[255, 255, 255, 255]);

        let mut shader_meta: ShaderMeta = shader::meta();
        shader_meta
            .uniforms
            .uniforms
            .push(UniformDesc::new("Model", UniformType::Mat4));
        shader_meta
            .uniforms
            .uniforms
            .push(UniformDesc::new("Projection", UniformType::Mat4));

        let shader = ctx
            .new_shader(
                match ctx.info().backend {
                    Backend::OpenGl => ShaderSource::Glsl {
                        vertex: shader::GL_VERTEX,
                        fragment: shader::GL_FRAGMENT,
                    },
                    Backend::Metal => ShaderSource::Msl {
                        program: shader::METAL,
                    },
                },
                shader_meta,
            )
            .unwrap();

        let params = PipelineParams {
            color_blend: Some(BlendState::new(
                Equation::Add,
                BlendFactor::Value(BlendValue::SourceAlpha),
                BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
            )),
            ..Default::default()
        };

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float2),
                VertexAttribute::new("in_color", VertexFormat::Float4),
                VertexAttribute::new("in_uv", VertexFormat::Float2),
            ],
            shader,
            params,
        );

        // Polygons must have counter-clockwise orientation

        //    0             1
        // (-1, 1) ----- (1, 1)
        //    |          /  |
        //    |        /    |
        //    |      /      |
        //    |    /        |
        // (-1, -1) ---- (1, -1)
        //    2             3
        //
        // faces: 021, 123
        /*
        let layer1 = Layer {
            model: glam::Mat4::IDENTITY,
            verts: vec![
                // top left
                Vertex {
                    pos: [-0.2, 0.2],
                    color: [1., 0., 1., 1.],
                    uv: [0., 0.],
                },
                // top right
                Vertex {
                    pos: [0.2, 0.2],
                    color: [1., 1., 0., 1.],
                    uv: [1., 0.],
                },
                // bottom left
                Vertex {
                    pos: [-0.2, -0.2],
                    color: [0., 0., 0.8, 1.],
                    uv: [0., 1.],
                },
                // bottom right
                Vertex {
                    pos: [0.2, -0.2],
                    color: [1., 1., 0., 1.],
                    uv: [1., 1.],
                },
            ],
            idxs: vec![0, 2, 1, 1, 2, 3]
        };

        let layer2 = Layer {
            model: glam::Mat4::IDENTITY,
            verts: vec![
                // top left
                Vertex {
                    pos: [-0.2, 0.2],
                    color: [0., 1., 1., 1.],
                    uv: [0., 0.],
                },
                // top right
                Vertex {
                    pos: [0.2, 0.2],
                    color: [0.2, 0.8, 1., 1.],
                    uv: [1., 0.],
                },
                // bottom left
                Vertex {
                    pos: [-0.2, -0.2],
                    color: [0.1, 0.2, 0.2, 1.],
                    uv: [0., 1.],
                },
                // bottom right
                Vertex {
                    pos: [0.2, -0.2],
                    color: [0., 0.6, 0.1, 1.],
                    uv: [1., 1.],
                },
            ],
            idxs: vec![0, 2, 1, 1, 2, 3]
        };
        */

        let mut stage = Stage {
            ctx,
            pipeline,
            white_texture,
            proj: glam::Mat4::IDENTITY,
            layers: HashMap::new(),
            recv_req,
            send_res,
            iface_ref,
        };
        //stage.layers.insert("box1".to_string(), layer1);
        //stage.layers.insert("box2".to_string(), layer2);
        #[rustfmt::skip]
        stage.draw_line(
            "origin".to_string(),
            -0.1, 0., 0.1, 0.,
            0.001,
            1., 0., 0., 1.,
        );
        #[rustfmt::skip]
        stage.draw_line(
            "origin".to_string(),
            0., 0.1, 0., -0.1,
            0.001,
            1., 0., 0., 1.,
        );
        stage
    }

    fn handle_cmd(&mut self, cmd: CommandRequest) {
        match cmd {
            CommandRequest::Hello => {
                debug!("hello()");
                self.send_res
                    .send(CommandResponse::Hello("hello".to_string()))
                    .unwrap();
            }
            CommandRequest::DrawLine(layer_name, x1, y1, x2, y2, thickness, r, g, b, a) => {
                self.draw_line(layer_name, x1, y1, x2, y2, thickness, r, g, b, a)
            }
            CommandRequest::Pan(x, y) => self.pan(x, y),
            CommandRequest::Zoom(scale) => self.zoom(scale),
            CommandRequest::ScreenToWorld(x, y) => {
                debug!("screen_to_world({}, {})", x, y);
                let (x, y) = self.screen_to_world(x, y);
                self.send_res
                    .send(CommandResponse::ScreenToWorld(x, y))
                    .unwrap();
            }
            CommandRequest::GetLayers => {
                debug!("get_layers()");
                let layer_names = self.layers.keys().cloned().collect();
                self.send_res
                    .send(CommandResponse::GetLayers(layer_names))
                    .unwrap();
            }
            CommandRequest::DeleteLayer(layer_name) => {
                debug!("delete_layer({})", layer_name);
                let is_success = self.layers.remove(&layer_name).is_some();
                self.send_res
                    .send(CommandResponse::DeleteLayer(is_success))
                    .unwrap();
            }
            CommandRequest::ShowLayer(layer_name) => {
                debug!("show_layer({})", layer_name);
                let is_success = match self.layers.get_mut(&layer_name) {
                    Some(layer) => {
                        layer.is_hidden = false;
                        true
                    }
                    None => false,
                };
                self.send_res
                    .send(CommandResponse::ShowLayer(is_success))
                    .unwrap();
            }
            CommandRequest::HideLayer(layer_name) => {
                debug!("hide_layer({})", layer_name);
                let is_success = match self.layers.get_mut(&layer_name) {
                    Some(layer) => {
                        layer.is_hidden = true;
                        true
                    }
                    None => false,
                };
                self.send_res
                    .send(CommandResponse::HideLayer(is_success))
                    .unwrap();
            }
            CommandRequest::SetLayerPos(layer_name, x, y) => {
                debug!("set_layer_pos({}, {}, {})", layer_name, x, y);
                let model = glam::Mat4::from_translation(glam::Vec3::new(x, y, 0.));
                let is_success = match self.layers.get_mut(&layer_name) {
                    Some(layer) => {
                        layer.model = model;
                        true
                    }
                    None => false,
                };
                self.send_res
                    .send(CommandResponse::SetLayerPos(is_success))
                    .unwrap();
            }
            CommandRequest::ScreenSize => {
                debug!("screen_size()");
                let (screen_width, screen_height) = window::screen_size();
                self.send_res
                    .send(CommandResponse::ScreenSize(screen_width, screen_height))
                    .unwrap();
            }
        }
    }

    #[rustfmt::skip]
    fn draw_line(
        &mut self, layer_name: String,
        x1: f32, y1: f32, x2: f32, y2: f32,
        thickness: f32,
        r: f32, g: f32, b: f32, a: f32,
    ) {
        debug!(
            "draw_line({}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
            layer_name, x1, y1, x2, y2, thickness, r, g, b, a
        );
        let color = [r, g, b, a];
        let (mut verts, mut idxs) = draw_line(x1, y1, x2, y2, thickness, color);
        let layer = self.layers.entry(layer_name).or_insert_with(Layer::new);
        let offset = layer.verts.len() as u32;
        layer.verts.append(&mut verts);
        for idx in &mut idxs {
            *idx += offset;
        }
        layer.idxs.append(&mut idxs);
    }

    fn pan(&mut self, x: f32, y: f32) {
        debug!("pan({}, {})", x, y);
        self.proj *= glam::Mat4::from_translation(glam::Vec3::new(x, y, 0.));
    }
    fn zoom(&mut self, scale: f32) {
        debug!("zoom({})", scale);
        self.proj *= glam::Mat4::from_scale(glam::Vec3::new(scale, scale, 1.));
    }

    fn calc_proj_matrix(&self) -> glam::Mat4 {
        let (screen_width, screen_height) = window::screen_size();
        // Preserve the same size irregardless of the screen size
        self.proj
            * glam::Mat4::from_scale(glam::Vec3::new(
                2500. / screen_width,
                2500. / screen_height,
                1.,
            ))
    }

    // Screen here refers to (0, 1)
    fn screen_to_world(&self, x: f32, y: f32) -> (f32, f32) {
        let x = 2. * x - 1.;
        let y = 1. - 2. * y;
        let pos = glam::vec4(x, y, 0., 1.);
        let proj_inv = self.calc_proj_matrix().inverse();
        let world_pos = (proj_inv * pos).xy();
        (world_pos.x, world_pos.y)
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {
        let start = time::Instant::now();
        while time::Instant::now() - start < time::Duration::from_millis(10) {
            // Only sleep for 10ms so we maintain 60 FPS
            match self.recv_req.recv_timeout(time::Duration::from_millis(10)) {
                Ok(cmd) => self.handle_cmd(cmd),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("Rx should not be disconnected!")
                }
            }
        }
    }

    // Only do drawing here. Apps might not call this when minimized.
    fn draw(&mut self) {
        let clear = PassAction::clear_color(0., 0., 0., 1.);
        self.ctx.begin_default_pass(clear);
        self.ctx.end_render_pass();

        let (screen_width, screen_height) = window::screen_size();
        //// Preserve the same size irregardless of the screen size
        //let proj = self.proj * glam::Mat4::from_scale(glam::Vec3::new(2500./screen_width, 2500./screen_height, 1.));
        let proj = self.calc_proj_matrix();

        for layer in self.layers.values() {
            if layer.is_hidden {
                continue
            }

            let vertex_buffer = self.ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&layer.verts),
            );

            let index_buffer = self.ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&layer.idxs),
            );

            let bindings = Bindings {
                vertex_buffers: vec![vertex_buffer],
                index_buffer: index_buffer,
                images: vec![self.white_texture],
            };

            //self.ctx.begin_default_pass(Default::default());
            self.ctx.begin_default_pass(PassAction::Nothing);

            self.ctx.apply_pipeline(&self.pipeline);
            self.ctx
                .apply_viewport(0, 0, screen_width as i32, screen_height as i32);
            self.ctx
                .apply_scissor_rect(0, 0, screen_width as i32, screen_height as i32);
            self.ctx.apply_bindings(&bindings);

            let mut uniforms_data = [0u8; 128];
            let data: [u8; 64] = unsafe { std::mem::transmute_copy(&layer.model) };
            uniforms_data[0..64].copy_from_slice(&data);
            let data: [u8; 64] = unsafe { std::mem::transmute_copy(&proj) };
            uniforms_data[64..].copy_from_slice(&data);
            assert_eq!(128, 2 * UniformType::Mat4.size());

            self.ctx
                .apply_uniforms_from_bytes(uniforms_data.as_ptr(), uniforms_data.len());

            self.ctx.draw(0, layer.idxs.len() as i32, 1);
            self.ctx.end_render_pass();
        }
        self.ctx.commit_frame();
    }

    fn key_down_event(&mut self, keycode: KeyCode, modifiers: KeyMods, repeat: bool) {
        let mut mods = vec![];
        if modifiers.shift {
            mods.push("shift");
        }
        if modifiers.ctrl {
            mods.push("ctrl");
        }
        if modifiers.alt {
            mods.push("alt");
        }
        if modifiers.logo {
            mods.push("logo");
        }

        let send_key_down = |key| {
            smol::future::block_on(DbusApi::key_down(
                self.iface_ref.signal_context(),
                key,
                mods,
                repeat,
            ))
            .expect("signal");
        };
        match keycode {
            KeyCode::Space => send_key_down("Space"),
            KeyCode::Apostrophe => send_key_down("Apostrophe"),
            KeyCode::Comma => send_key_down("Comma"),
            KeyCode::Minus => send_key_down("Minus"),
            KeyCode::Period => send_key_down("Period"),
            KeyCode::Slash => send_key_down("Slash"),
            KeyCode::Key0 => send_key_down("Key0"),
            KeyCode::Key1 => send_key_down("Key1"),
            KeyCode::Key2 => send_key_down("Key2"),
            KeyCode::Key3 => send_key_down("Key3"),
            KeyCode::Key4 => send_key_down("Key4"),
            KeyCode::Key5 => send_key_down("Key5"),
            KeyCode::Key6 => send_key_down("Key6"),
            KeyCode::Key7 => send_key_down("Key7"),
            KeyCode::Key8 => send_key_down("Key8"),
            KeyCode::Key9 => send_key_down("Key9"),
            KeyCode::Semicolon => send_key_down("Semicolon"),
            KeyCode::Equal => send_key_down("Equal"),
            KeyCode::A => send_key_down("A"),
            KeyCode::B => send_key_down("B"),
            KeyCode::C => send_key_down("C"),
            KeyCode::D => send_key_down("D"),
            KeyCode::E => send_key_down("E"),
            KeyCode::F => send_key_down("F"),
            KeyCode::G => send_key_down("G"),
            KeyCode::H => send_key_down("H"),
            KeyCode::I => send_key_down("I"),
            KeyCode::J => send_key_down("J"),
            KeyCode::K => send_key_down("K"),
            KeyCode::L => send_key_down("L"),
            KeyCode::M => send_key_down("M"),
            KeyCode::N => send_key_down("N"),
            KeyCode::O => send_key_down("O"),
            KeyCode::P => send_key_down("P"),
            KeyCode::Q => send_key_down("Q"),
            KeyCode::R => send_key_down("R"),
            KeyCode::S => send_key_down("S"),
            KeyCode::T => send_key_down("T"),
            KeyCode::U => send_key_down("U"),
            KeyCode::V => send_key_down("V"),
            KeyCode::W => send_key_down("W"),
            KeyCode::X => send_key_down("X"),
            KeyCode::Y => send_key_down("Y"),
            KeyCode::Z => send_key_down("Z"),
            KeyCode::LeftBracket => send_key_down("LeftBracket"),
            KeyCode::Backslash => send_key_down("Backslash"),
            KeyCode::RightBracket => send_key_down("RightBracket"),
            KeyCode::GraveAccent => send_key_down("GraveAccent"),
            KeyCode::World1 => send_key_down("World1"),
            KeyCode::World2 => send_key_down("World2"),
            KeyCode::Escape => send_key_down("Escape"),
            KeyCode::Enter => send_key_down("Enter"),
            KeyCode::Tab => send_key_down("Tab"),
            KeyCode::Backspace => send_key_down("Backspace"),
            KeyCode::Insert => send_key_down("Insert"),
            KeyCode::Delete => send_key_down("Delete"),
            KeyCode::Right => send_key_down("Right"),
            KeyCode::Left => send_key_down("Left"),
            KeyCode::Down => send_key_down("Down"),
            KeyCode::Up => send_key_down("Up"),
            KeyCode::PageUp => send_key_down("PageUp"),
            KeyCode::PageDown => send_key_down("PageDown"),
            KeyCode::Home => send_key_down("Home"),
            KeyCode::End => send_key_down("End"),
            KeyCode::CapsLock => send_key_down("CapsLock"),
            KeyCode::ScrollLock => send_key_down("ScrollLock"),
            KeyCode::NumLock => send_key_down("NumLock"),
            KeyCode::PrintScreen => send_key_down("PrintScreen"),
            KeyCode::Pause => send_key_down("Pause"),
            KeyCode::F1 => send_key_down("F1"),
            KeyCode::F2 => send_key_down("F2"),
            KeyCode::F3 => send_key_down("F3"),
            KeyCode::F4 => send_key_down("F4"),
            KeyCode::F5 => send_key_down("F5"),
            KeyCode::F6 => send_key_down("F6"),
            KeyCode::F7 => send_key_down("F7"),
            KeyCode::F8 => send_key_down("F8"),
            KeyCode::F9 => send_key_down("F9"),
            KeyCode::F10 => send_key_down("F10"),
            KeyCode::F11 => send_key_down("F11"),
            KeyCode::F12 => send_key_down("F12"),
            KeyCode::F13 => send_key_down("F13"),
            KeyCode::F14 => send_key_down("F14"),
            KeyCode::F15 => send_key_down("F15"),
            KeyCode::F16 => send_key_down("F16"),
            KeyCode::F17 => send_key_down("F17"),
            KeyCode::F18 => send_key_down("F18"),
            KeyCode::F19 => send_key_down("F19"),
            KeyCode::F20 => send_key_down("F20"),
            KeyCode::F21 => send_key_down("F21"),
            KeyCode::F22 => send_key_down("F22"),
            KeyCode::F23 => send_key_down("F23"),
            KeyCode::F24 => send_key_down("F24"),
            KeyCode::F25 => send_key_down("F25"),
            KeyCode::Kp0 => send_key_down("Kp0"),
            KeyCode::Kp1 => send_key_down("Kp1"),
            KeyCode::Kp2 => send_key_down("Kp2"),
            KeyCode::Kp3 => send_key_down("Kp3"),
            KeyCode::Kp4 => send_key_down("Kp4"),
            KeyCode::Kp5 => send_key_down("Kp5"),
            KeyCode::Kp6 => send_key_down("Kp6"),
            KeyCode::Kp7 => send_key_down("Kp7"),
            KeyCode::Kp8 => send_key_down("Kp8"),
            KeyCode::Kp9 => send_key_down("Kp9"),
            KeyCode::KpDecimal => send_key_down("KpDecimal"),
            KeyCode::KpDivide => send_key_down("KpDivide"),
            KeyCode::KpMultiply => send_key_down("KpMultiply"),
            KeyCode::KpSubtract => send_key_down("KpSubtract"),
            KeyCode::KpAdd => send_key_down("KpAdd"),
            KeyCode::KpEnter => send_key_down("KpEnter"),
            KeyCode::KpEqual => send_key_down("KpEqual"),
            KeyCode::LeftShift => send_key_down("LeftShift"),
            KeyCode::LeftControl => send_key_down("LeftControl"),
            KeyCode::LeftAlt => send_key_down("LeftAlt"),
            KeyCode::LeftSuper => send_key_down("LeftSuper"),
            KeyCode::RightShift => send_key_down("RightShift"),
            KeyCode::RightControl => send_key_down("RightControl"),
            KeyCode::RightAlt => send_key_down("RightAlt"),
            KeyCode::RightSuper => send_key_down("RightSuper"),
            KeyCode::Menu => send_key_down("Menu"),
            KeyCode::Unknown => send_key_down("Unknown"),
        }

        /*
        match keycode {
            KeyCode::Left => {
                let layer2 = &mut self.layers.get_mut(&"layer2".to_string()).unwrap();
                layer2.model *= glam::Mat4::from_translation(glam::Vec3::new(-0.1, 0., 0.));
            }
            KeyCode::Right => {
                let layer2 = &mut self.layers.get_mut(&"layer2".to_string()).unwrap();
                layer2.model *= glam::Mat4::from_translation(glam::Vec3::new(0.1, 0., 0.));
            }
            KeyCode::Up => {
                let layer2 = &mut self.layers.get_mut(&"layer2".to_string()).unwrap();
                layer2.model *= glam::Mat4::from_translation(glam::Vec3::new(0., 0.1, 0.));
            }
            KeyCode::Down => {
                let layer2 = &mut self.layers.get_mut(&"layer2".to_string()).unwrap();
                layer2.model *= glam::Mat4::from_translation(glam::Vec3::new(0., -0.1, 0.));
            }
            KeyCode::L => {
                self.proj *= glam::Mat4::from_translation(glam::Vec3::new(-0.1, 0., 0.));
            }
            KeyCode::H => {
                self.proj *= glam::Mat4::from_translation(glam::Vec3::new(0.1, 0., 0.));
            }
            KeyCode::J => {
                self.proj *= glam::Mat4::from_translation(glam::Vec3::new(0., 0.1, 0.));
            }
            KeyCode::K => {
                self.proj *= glam::Mat4::from_translation(glam::Vec3::new(0., -0.1, 0.));
            }
            KeyCode::Z => {
                self.proj *= glam::Mat4::from_scale(glam::Vec3::new(0.9, 0.9, 0.9));
            }
            KeyCode::X => {
                self.proj *= glam::Mat4::from_scale(glam::Vec3::new(1.1, 1.1, 1.1));
            }
            _ => {}
        }
        */
    }
    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        let (screen_width, screen_height) = window::screen_size();
        let (x, y) = (x / screen_width, y / screen_height);
        let (x, y) = self.screen_to_world(x, y);
        smol::future::block_on(DbusApi::mouse_motion(self.iface_ref.signal_context(), x, y))
            .expect("signal");
        //println!("{} {}", x, y);
    }
    fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        smol::future::block_on(DbusApi::mouse_wheel(self.iface_ref.signal_context(), x, y))
            .expect("signal");
        //let scale = 1.0 + y/10.;
        //self.proj *= glam::Mat4::from_scale(glam::Vec3::new(scale, scale, scale));
    }
    fn mouse_button_down_event(&mut self, button: MouseButton, x: f32, y: f32) {
        let (screen_width, screen_height) = window::screen_size();
        let (x, y) = (x / screen_width, y / screen_height);
        let (x, y) = self.screen_to_world(x, y);
        smol::future::block_on(DbusApi::mouse_button_down(
            self.iface_ref.signal_context(),
            button.to_str(),
            x,
            y,
        ))
        .expect("signal");
        //window::show_keyboard(true);
        //println!("{:?} {} {}", button, x, y);
    }
    fn mouse_button_up_event(&mut self, button: MouseButton, x: f32, y: f32) {
        let (screen_width, screen_height) = window::screen_size();
        let (x, y) = (x / screen_width, y / screen_height);
        let (x, y) = self.screen_to_world(x, y);
        smol::future::block_on(DbusApi::mouse_button_up(
            self.iface_ref.signal_context(),
            button.to_str(),
            x,
            y,
        ))
        .expect("signal");
    }

    fn resize_event(&mut self, width: f32, height: f32) {
        //self.proj *= glam::Mat4::from_scale(glam::Vec3::new(aspect_ratio, 1., 1.));
        //debug!("resize! {} {}", width, height);
    }
}

fn draw_line(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    thickness: f32,
    color: [f32; 4],
) -> (Vec<Vertex>, Vec<u32>) {
    let dx = x2 - x1;
    let dy = y2 - y1;

    // https://stackoverflow.com/questions/1243614/how-do-i-calculate-the-normal-vector-of-a-line-segment

    let nx = -dy;
    let ny = dx;

    let tlen = (nx * nx + ny * ny).sqrt() / (thickness * 0.5);
    // This is an error but lets be more forgiving
    //assert!(tlen >= f32::EPSILON);
    if tlen < f32::EPSILON {
        return (vec![], vec![]);
    }
    let tx = nx / tlen;
    let ty = ny / tlen;

    // We don't care about UV coords
    let uv = [0f32; 2];

    #[rustfmt::skip]
    (
        vec![
            Vertex { pos: [x1 + tx, y1 + ty], color, uv, },
            Vertex { pos: [x1 - tx, y1 - ty], color, uv, },
            Vertex { pos: [x2 + tx, y2 + ty], color, uv, },
            Vertex { pos: [x2 - tx, y2 - ty], color, uv, },
        ],
        vec![0, 1, 2, 2, 1, 3],
    )
}

struct DbusApi {
    send_req: mpsc::Sender<CommandRequest>,
    recv_res: Arc<Mutex<mpsc::Receiver<CommandResponse>>>,
}

#[zbus::interface(name = "org.therapy.Therapy")]
impl DbusApi {
    async fn say_hello(&self) -> String {
        let res = dbus_request(self, CommandRequest::Hello);
        match res {
            CommandResponse::Hello(res) => res,
            _ => panic!("unexpected result!"),
        }
    }

    async fn draw_line(
        &self,
        layer_name: String,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        thickness: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) {
        let cmd = CommandRequest::DrawLine(layer_name, x1, y1, x2, y2, thickness, r, g, b, a);
        self.send_req.send(cmd).unwrap();
    }

    async fn pan(&self, x: f32, y: f32) {
        let cmd = CommandRequest::Pan(x, y);
        self.send_req.send(cmd).unwrap();
    }

    async fn zoom(&self, scale: f32) {
        let cmd = CommandRequest::Zoom(scale);
        self.send_req.send(cmd).unwrap();
    }

    async fn screen_to_world(&self, x: f32, y: f32) -> (f32, f32) {
        let res = dbus_request(self, CommandRequest::ScreenToWorld(x, y));
        match res {
            CommandResponse::ScreenToWorld(x, y) => (x, y),
            _ => panic!("unexpected result!"),
        }
    }

    async fn get_layers(&self) -> Vec<String> {
        let res = dbus_request(self, CommandRequest::GetLayers);
        match res {
            CommandResponse::GetLayers(layer_names) => layer_names,
            _ => panic!("unexpected result!"),
        }
    }

    async fn delete_layer(&self, layer_name: String) -> bool {
        let res = dbus_request(self, CommandRequest::DeleteLayer(layer_name));
        match res {
            CommandResponse::DeleteLayer(is_success) => is_success,
            _ => panic!("unexpected result!"),
        }
    }

    async fn show_layer(&self, layer_name: String) -> bool {
        let res = dbus_request(self, CommandRequest::ShowLayer(layer_name));
        match res {
            CommandResponse::ShowLayer(is_success) => is_success,
            _ => panic!("unexpected result!"),
        }
    }

    async fn hide_layer(&self, layer_name: String) -> bool {
        let res = dbus_request(self, CommandRequest::HideLayer(layer_name));
        match res {
            CommandResponse::HideLayer(is_success) => is_success,
            _ => panic!("unexpected result!"),
        }
    }

    async fn set_layer_pos(&self, layer_name: String, x: f32, y: f32) -> bool {
        let res = dbus_request(self, CommandRequest::SetLayerPos(layer_name, x, y));
        match res {
            CommandResponse::SetLayerPos(is_success) => is_success,
            _ => panic!("unexpected result!"),
        }
    }

    async fn screen_size(&self) -> (f32, f32) {
        let res = dbus_request(self, CommandRequest::ScreenSize);
        match res {
            CommandResponse::ScreenSize(w, h) => (w, h),
            _ => panic!("unexpected result!"),
        }
    }

    #[zbus(signal)]
    async fn key_down(
        ctxt: &zbus::SignalContext<'_>,
        key: &str,
        keymods: Vec<&str>,
        repeat: bool,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn mouse_motion(ctxt: &zbus::SignalContext<'_>, x: f32, y: f32) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn mouse_wheel(ctxt: &zbus::SignalContext<'_>, x: f32, y: f32) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn mouse_button_down(
        ctxt: &zbus::SignalContext<'_>,
        button: &str,
        x: f32,
        y: f32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn mouse_button_up(
        ctxt: &zbus::SignalContext<'_>,
        button: &str,
        x: f32,
        y: f32,
    ) -> zbus::Result<()>;
}

fn dbus_request(api: &DbusApi, cmd: CommandRequest) -> CommandResponse {
    api.send_req.send(cmd).unwrap();
    let recv_res = api.recv_res.lock().unwrap();
    match recv_res.recv_timeout(std::time::Duration::from_millis(400)) {
        Ok(cmd) => cmd,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("Rx didn't receive expected response!")
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("Rx should not be disconnected!")
        }
    }
}

async fn setup_dbus(connection: &zbus::Connection, dbus_api: DbusApi) -> BoxResult {
    // setup the server
    connection
        .object_server()
        .at("/org/therapy/Therapy", dbus_api)
        .await?;
    // before requesting the name
    connection.request_name("org.therapy.Therapy").await?;

    Ok(())
}

fn main() -> BoxResult {
    //let ex = std::sync::Arc::new(smol::Executor::new());
    //let (signal, shutdown) = smol::channel::unbounded::<()>();

    //let (_, result) = easy_parallel::Parallel::new()
    //    // Run four executor threads.
    //    .each(0..4, |_| smol::future::block_on(ex.run(shutdown.recv())))
    //    // Run the main future on the current thread.
    //    .finish(|| smol::future::block_on(async {
    //        let connection = zbus::Connection::session().await?;
    //        setup_dbus(&connection).await?;
    //        gui_main();
    //        drop(signal);
    //        Ok(())
    //    }));
    //result

    smol::future::block_on(async {
        let (send_req, recv_req) = mpsc::channel();
        let (send_res, recv_res) = mpsc::channel();

        let dbus_api = DbusApi {
            send_req,
            recv_res: Arc::new(Mutex::new(recv_res)),
        };
        let connection = zbus::Connection::session().await?;
        setup_dbus(&connection, dbus_api).await?;

        let object_server = connection.object_server();
        let iface_ref = object_server
            .interface::<_, DbusApi>("/org/therapy/Therapy")
            .await?;
        gui_main(recv_req, send_res, iface_ref);

        Ok(())
    })
}

fn gui_main(
    recv_req: mpsc::Receiver<CommandRequest>,
    send_res: mpsc::Sender<CommandResponse>,
    iface_ref: zbus::InterfaceRef<DbusApi>,
) {
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(LevelFilter::Debug)
                .with_tag("fagman"),
        );
    }

    #[cfg(target_os = "linux")]
    {
        let term_logger = simplelog::TermLogger::new(
            simplelog::LevelFilter::Debug,
            simplelog::Config::default(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        );
        simplelog::CombinedLogger::init(vec![term_logger]).expect("logger");
    }

    let mut conf = miniquad::conf::Conf {
        high_dpi: true,
        window_resizable: true,
        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::WaylandWithX11Fallback,
            wayland_use_fallback_decorations: false,
            ..Default::default()
        },
        ..Default::default()
    };
    let metal = std::env::args().nth(1).as_deref() == Some("metal");
    conf.platform.apple_gfx_api = if metal {
        conf::AppleGfxApi::Metal
    } else {
        conf::AppleGfxApi::OpenGl
    };

    miniquad::start(conf, || {
        window::show_keyboard(true);
        Box::new(Stage::new(recv_req, send_res, iface_ref))
    });
}

mod shader {
    use miniquad::*;

    pub const GL_VERTEX: &str = r#"#version 100
    attribute vec2 in_pos;
    attribute vec4 in_color;
    attribute vec2 in_uv;

    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform mat4 Model;
    uniform mat4 Projection;

    void main() {
        gl_Position = Projection * Model * vec4(in_pos, 0, 1);
        color = in_color;
        uv = in_uv;
    }"#;

    pub const GL_FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform sampler2D tex;

    void main() {
        gl_FragColor = color * texture2D(tex, uv);
    }"#;

    pub const METAL: &str = r#"
    #include <metal_stdlib>

    using namespace metal;

    struct Uniforms
    {
        float4x4 Model;
        float4x4 Projection;
    };

    struct Vertex
    {
        float2 in_pos   [[attribute(0)]];
        float4 in_color [[attribute(1)]];
        float2 in_uv    [[attribute(2)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float4 color [[user(locn0)]];
        float2 uv [[user(locn1)]];
    };

    vertex RasterizerData vertexShader(Vertex v [[stage_in]])
    {
        RasterizerData out;

        out.position = uniforms.Model * uniforms.Projection * float4(v.in_pos.xy, 0.0, 1.0);
        out.color = v.in_color;
        out.uv = v.texcoord;

        return out;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]], texture2d<float> tex [[texture(0)]], sampler texSmplr [[sampler(0)]])
    {
        return in.color * tex.sample(texSmplr, in.uv);
    }

    "#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout { uniforms: vec![] },
        }
    }
}

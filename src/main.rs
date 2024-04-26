#![feature(stmt_expr_attributes)]

use darkfi_serial::{deserialize, Decodable, Encodable, SerialDecodable};
use glam::Vec4Swizzles;
use miniquad::*;
use std::{collections::HashMap, fmt, io::Cursor, time};

#[macro_use]
extern crate log;
#[allow(unused_imports)]
use log::LevelFilter;

#[repr(u8)]
enum Command {
    Hello = 0,
    DrawLine = 1,
    Pan = 2,
    Zoom = 3,
    ScreenToWorld = 4,
    GetLayers = 5,
    DeleteLayer = 6,
    ShowLayer = 7,
    HideLayer = 8,
    SetLayerPos = 9,
    ScreenSize = 10,
}

impl Command {
    // Ridiculous
    fn from_u8(cmd: u8) -> Self {
        match cmd {
            0 => Command::Hello,
            1 => Command::DrawLine,
            2 => Command::Pan,
            3 => Command::Zoom,
            4 => Command::ScreenToWorld,
            5 => Command::GetLayers,
            6 => Command::DeleteLayer,
            7 => Command::ShowLayer,
            8 => Command::HideLayer,
            9 => Command::SetLayerPos,
            10 => Command::ScreenSize,
            _ => panic!("invalid cmd"),
        }
    }
}

#[derive(Debug, SerialDecodable)]
#[rustfmt::skip]
struct RequestDrawLine {
    layer_name: String,
    x1: f32, y1: f32, x2: f32, y2: f32,
    thickness: f32,
    r: f32, g: f32, b: f32, a: f32
}

#[repr(u8)]
enum PubEvents {
    KeyDown = 0,
    MouseMotion = 1,
    MouseWheel = 2,
    MouseButtonDown = 3,
    MouseButtonUp = 4,
}

trait MouseButtonAsString {
    fn to_u8(&self) -> u8;
}

impl MouseButtonAsString for MouseButton {
    fn to_u8(&self) -> u8 {
        match self {
            MouseButton::Left => 0,
            MouseButton::Middle => 1,
            MouseButton::Right => 2,
            MouseButton::Unknown => 3,
        }
    }
}

#[repr(C)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 4],
    uv: [f32; 2],
}

#[repr(C)]
struct Face {
    idxs: [u32; 3],
}

impl fmt::Debug for Face {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.idxs)
    }
}

struct Layer {
    model: glam::Mat4,
    verts: Vec<Vertex>,
    faces: Vec<Face>,
    is_hidden: bool,
}

impl Layer {
    fn new() -> Self {
        Self {
            model: glam::Mat4::IDENTITY,
            verts: vec![],
            faces: vec![],
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
    // req-reply commands
    req_socket: zmq::Socket,
    // events from this canvas
    pub_socket: zmq::Socket,
    // low latency no reply commands
    sub_socket: zmq::Socket,
}

impl Stage {
    pub fn new() -> Stage {
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
        let zmq_ctx = zmq::Context::new();
        let req_socket = zmq_ctx.socket(zmq::REP).unwrap();
        req_socket.bind("tcp://*:9464").unwrap();
        let pub_socket = zmq_ctx.socket(zmq::PUB).unwrap();
        pub_socket.bind("tcp://*:9465").unwrap();
        let sub_socket = zmq_ctx.socket(zmq::SUB).unwrap();
        sub_socket.set_subscribe(b"").unwrap();
        sub_socket.bind("tcp://*:9466").unwrap();

        let mut stage = Stage {
            ctx,
            pipeline,
            white_texture,
            proj: glam::Mat4::IDENTITY,
            layers: HashMap::new(),
            req_socket,
            pub_socket,
            sub_socket
        };
        //stage.layers.insert("box1".to_string(), layer1);
        //stage.layers.insert("box2".to_string(), layer2);
        #[rustfmt::skip]
        stage.draw_line(
            "origin".to_string(),
            -0.1, 0., 0.1, 0.,
            0.001,
            1., 0., 0., 0.4,
        );
        #[rustfmt::skip]
        stage.draw_line(
            "origin".to_string(),
            0., 0.1, 0., -0.1,
            0.001,
            1., 0., 0., 0.4,
        );
        stage
    }

    #[rustfmt::skip]
    fn draw_line(
        &mut self, layer_name: String,
        x1: f32, y1: f32, x2: f32, y2: f32,
        thickness: f32,
        r: f32, g: f32, b: f32, a: f32,
    ) {
        //debug!(
        //    "draw_line({}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
        //    layer_name, x1, y1, x2, y2, thickness, r, g, b, a
        //);
        let color = [r, g, b, a];
        let (mut verts, mut faces) = draw_line(x1, y1, x2, y2, thickness, color);
        let layer = self.layers.entry(layer_name).or_insert_with(Layer::new);
        let offset = layer.verts.len() as u32;
        layer.verts.append(&mut verts);
        for face in &mut faces {
            for idx in &mut face.idxs {
                *idx += offset;
            }
        }
        layer.faces.append(&mut faces);
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

    fn process_req(&mut self) {
        let req = self.req_socket.recv_multipart(zmq::DONTWAIT).unwrap();

        assert_eq!(req[0].len(), 1);
        assert_eq!(req.len(), 2);
        let cmd = Command::from_u8(req[0][0]);
        let payload = req[1].clone();

        let mut reply = vec![];

        match cmd {
            Command::Hello => {
                assert_eq!(payload.len(), 0);
                "hello".encode(&mut reply).unwrap();
            }
            Command::DrawLine | Command::Pan | Command::Zoom => {
                panic!("use sub socket instead!");
            }
            Command::ScreenToWorld => {
                let mut cur = Cursor::new(payload);
                let x = f32::decode(&mut cur).unwrap();
                let y = f32::decode(cur).unwrap();
                //debug!("screen_to_world({}, {})", x, y);
                let (x, y) = self.screen_to_world(x, y);
                x.encode(&mut reply).unwrap();
                y.encode(&mut reply).unwrap();
            }
            Command::GetLayers => {
                debug!("get_layers()");
                let layer_names: Vec<String> = self.layers.keys().cloned().collect();
                layer_names.encode(&mut reply).unwrap();
            }
            Command::DeleteLayer => {
                let layer_name: String = deserialize(&payload).unwrap();
                debug!("delete_layer({})", layer_name);
                let is_success = self.layers.remove(&layer_name).is_some();
                is_success.encode(&mut reply).unwrap();
            }
            Command::ShowLayer => {
                let layer_name: String = deserialize(&payload).unwrap();
                debug!("show_layer({})", layer_name);
                let is_success = match self.layers.get_mut(&layer_name) {
                    Some(layer) => {
                        layer.is_hidden = false;
                        true
                    }
                    None => false,
                };
                is_success.encode(&mut reply).unwrap();
            }
            Command::HideLayer => {
                let layer_name: String = deserialize(&payload).unwrap();
                debug!("hide_layer({})", layer_name);
                let is_success = match self.layers.get_mut(&layer_name) {
                    Some(layer) => {
                        layer.is_hidden = true;
                        true
                    }
                    None => false,
                };
                is_success.encode(&mut reply).unwrap();
            }
            Command::SetLayerPos => {
                let mut cur = Cursor::new(payload);
                let layer_name = String::decode(&mut cur).unwrap();
                let x = f32::decode(&mut cur).unwrap();
                let y = f32::decode(cur).unwrap();
                //debug!("set_layer_pos({}, {}, {})", layer_name, x, y);
                let model = glam::Mat4::from_translation(glam::Vec3::new(x, y, 0.));
                let is_success = match self.layers.get_mut(&layer_name) {
                    Some(layer) => {
                        layer.model = model;
                        true
                    }
                    None => false,
                };
                is_success.encode(&mut reply).unwrap();
            }
            Command::ScreenSize => {
                debug!("screen_size()");
                let (screen_width, screen_height) = window::screen_size();
                screen_width.encode(&mut reply).unwrap();
                screen_height.encode(&mut reply).unwrap();
            }
        }

        self.req_socket.send(reply, 0).unwrap();
    }

    fn process_sub(&mut self) {
        let req = self.sub_socket.recv_multipart(zmq::DONTWAIT).unwrap();

        assert_eq!(req[0].len(), 1);
        assert_eq!(req.len(), 2);
        let cmd = Command::from_u8(req[0][0]);
        let payload = req[1].clone();

        match cmd {
            Command::DrawLine => {
                let params: RequestDrawLine = deserialize(&payload).unwrap();
                //debug!("draw_line({:?})", params);
                self.draw_line(
                    params.layer_name,
                    params.x1,
                    params.y1,
                    params.x2,
                    params.y2,
                    params.thickness,
                    params.r,
                    params.g,
                    params.b,
                    params.a,
                )
            }
            Command::Pan => {
                //let params: RequestPan = deserialize(&payload).unwrap();
                let mut cur = Cursor::new(payload);
                let x = f32::decode(&mut cur).unwrap();
                let y = f32::decode(cur).unwrap();
                debug!("pan({}, {})", x, y);
                self.pan(x, y)
            }
            Command::Zoom => {
                let scale: f32 = deserialize(&payload).unwrap();
                debug!("zoom({})", scale);
                self.zoom(scale)
            }
            _ => {
                panic!("only for no reply messages!")
            }
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {
        let instant = time::Instant::now();
        let mut remaining = 20i64;
        loop {
            // https://github.com/johnliu55tw/rust-zmq-poller/blob/master/src/main.rs
            let mut items = [
                self.req_socket.as_poll_item(zmq::POLLIN),
                self.sub_socket.as_poll_item(zmq::POLLIN),
            ];
            let _rc = zmq::poll(&mut items, remaining).unwrap();

            // Rust borrow checker things
            let (is_item0_readable, is_item1_readable) = (items[0].is_readable(), items[1].is_readable());
            drop(items);

            if is_item0_readable {
                self.process_req()
            }
            if is_item1_readable {
                self.process_sub()
            }

            remaining -= instant.elapsed().as_millis() as i64;
            if remaining <= 0 {
                break;
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
                continue;
            }

            let vertex_buffer = self.ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&layer.verts),
            );

            let bufsrc = unsafe {
                BufferSource::pointer(
                    layer.faces.as_ptr() as _,
                    std::mem::size_of_val(&layer.faces[..]),
                    std::mem::size_of::<u32>(),
                )
            };

            let index_buffer =
                self.ctx
                    .new_buffer(BufferType::IndexBuffer, BufferUsage::Immutable, bufsrc);

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

            self.ctx.draw(0, 3 * layer.faces.len() as i32, 1);
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

        let send_key_down = |key: &str| {
            let mut event = vec![];
            (PubEvents::KeyDown as u8).encode(&mut event).unwrap();
            key.encode(&mut event).unwrap();
            mods.encode(&mut event).unwrap();
            repeat.encode(&mut event).unwrap();
            //debug!("KeyDown => ({}, {:?}, {})", key, mods, repeat);
            self.pub_socket.send(event, 0).unwrap();
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
    }
    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        let (screen_width, screen_height) = window::screen_size();
        let (x, y) = (x / screen_width, y / screen_height);
        let (x, y) = self.screen_to_world(x, y);

        let mut event = vec![];
        (PubEvents::MouseMotion as u8).encode(&mut event).unwrap();
        x.encode(&mut event).unwrap();
        y.encode(&mut event).unwrap();
        //debug!("MouseMotion => ({}, {})", x, y);
        self.pub_socket.send(event, 0).unwrap();
    }
    fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        let mut event = vec![];
        (PubEvents::MouseWheel as u8).encode(&mut event).unwrap();
        x.encode(&mut event).unwrap();
        y.encode(&mut event).unwrap();
        debug!("MouseWheel => ({}, {})", x, y);
        self.pub_socket.send(event, 0).unwrap();

        //let scale = 1.0 + y/10.;
        //self.proj *= glam::Mat4::from_scale(glam::Vec3::new(scale, scale, scale));
    }
    fn mouse_button_down_event(&mut self, button: MouseButton, x: f32, y: f32) {
        let (screen_width, screen_height) = window::screen_size();
        let (x, y) = (x / screen_width, y / screen_height);
        let (x, y) = self.screen_to_world(x, y);

        let mut event = vec![];
        (PubEvents::MouseButtonDown as u8)
            .encode(&mut event)
            .unwrap();
        button.to_u8().encode(&mut event).unwrap();
        x.encode(&mut event).unwrap();
        y.encode(&mut event).unwrap();
        debug!("MouseButtonDown => ({:?}, {}, {})", button, x, y);
        self.pub_socket.send(event, 0).unwrap();
    }
    fn mouse_button_up_event(&mut self, button: MouseButton, x: f32, y: f32) {
        let (screen_width, screen_height) = window::screen_size();
        let (x, y) = (x / screen_width, y / screen_height);
        let (x, y) = self.screen_to_world(x, y);

        let mut event = vec![];
        (PubEvents::MouseButtonUp as u8).encode(&mut event).unwrap();
        button.to_u8().encode(&mut event).unwrap();
        x.encode(&mut event).unwrap();
        y.encode(&mut event).unwrap();
        debug!("MouseButtonUp => ({:?}, {}, {})", button, x, y);
        self.pub_socket.send(event, 0).unwrap();
    }
}

fn draw_line(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    thickness: f32,
    color: [f32; 4],
) -> (Vec<Vertex>, Vec<Face>) {
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
        vec![
            Face { idxs: [0, 1, 2] },
            Face { idxs: [2, 1, 3] }
        ],
    )
}

fn main() {
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

    miniquad::start(conf, || Box::new(Stage::new()));
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

#[deny(warnings)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate glium;
extern crate env_logger;
extern crate lights;

use std::error::Error;
use std::io::prelude::*;

use env_logger::LogBuilder;
use glium::backend::glutin_backend::GlutinFacade;
use glium::index::{NoIndices, PrimitiveType};
use glium::{Surface, VertexBuffer, DrawError, Program, DrawParameters, Depth};
use glium::glutin::Event;
use glium::texture::cubemap::Cubemap;

use lights::{App, Api, Painter, load_program, Camera, load_cubemap, Model};
use lights::math::*;

mod vertex;
mod models;

use vertex::{Vertex};

fn init_log() {
    LogBuilder::new()
        .parse("info")
        .init()
        .expect("Failed to init the logger");
}


fn main() {
    if let Err(e) = run() {
        writeln!(std::io::stderr(), "{}\n=(", e).unwrap();
        writeln!(std::io::stderr(), "\nGuru meditation {:#?}", e).unwrap();
    }
}

fn run() -> Result<(), Box<std::error::Error>> {
    init_log();
    try!(App::<Matisse>::run());
    Ok(())
}

struct Matisse {
    camera: Camera,
    skybox: SkyBox,
    bunny: Model,
    cube: Model,
    program: Program,
}

impl Painter for Matisse {
    fn new(facade: &GlutinFacade) -> Result<Matisse, Box<Error>> {

        Ok(Matisse {
            camera: Camera::new(vec3(0.0, 0.0, 3.0), vec3(0.0, 0.0, 0.0), Y),
            skybox: try!(SkyBox::new(facade)),
            bunny: try!(Model::load(facade, "bunny_with_normals.obj")),
            cube: try!(Model::load(facade, "cube.obj")),
            program: try!(load_program(facade, "mirror/vertex.glsl", "mirror/fragment.glsl")),
        })
    }

    fn process_event(&mut self, event: Event, delta: f32) {
        self.camera.process_event(event, delta)
    }

    fn draw<S: Surface>(&self, api: &mut Api<S>) -> std::result::Result<(), DrawError> {
        try!(self.skybox.draw(api, self));
        let uniforms = uniform! {
            model: id().scale(5.0),
            view: self.camera.view(),
            projection: api.projection(),
            camera_position: self.camera.position(),
            skybox: &self.skybox.cubemap,
        };
        try!(self.bunny.draw(api, &self.program, &uniforms));

        let uniforms = uniform! {
            model: id().translate(vec3(0.0, -0.3, 0.0)),
            view: self.camera.view(),
            projection: api.projection(),
            camera_position: self.camera.position(),
            skybox: &self.skybox.cubemap,
        };
        try!(self.cube.draw(api, &self.program, &uniforms));

        Ok(())
    }
}

struct SkyBox {
    vertex_buffer: VertexBuffer<Vertex>,
    program: Program,
    cubemap: Cubemap,
}

impl SkyBox {
    fn new(facade: &GlutinFacade) -> Result<SkyBox, Box<Error>> {
        let shape = Vertex::many(models::skybox());
        Ok(SkyBox {
            vertex_buffer: try!(VertexBuffer::new(facade, &shape)),
            program: try!(load_program(facade, "skybox/vertex.glsl", "skybox/fragment.glsl")),
            cubemap: try!(load_cubemap(facade, "skybox")),
        })
    }

    fn draw<S: Surface>(&self,
                        api: &mut Api<S>,
                        p: &Matisse)
                        -> std::result::Result<(), DrawError> {
        let uniforms = uniform! {
            view: p.camera.view(),
            projection: api.projection(),
            skybox: &self.cubemap,
        };

        try!(api.surface.draw(&self.vertex_buffer,
                              &NoIndices(PrimitiveType::TrianglesList),
                              &self.program,
                              &uniforms,
                              &DrawParameters {
                                  depth: Depth { write: false, ..Default::default() },
                                  ..api.default_params.clone()
                              }));
        Ok(())
    }
}

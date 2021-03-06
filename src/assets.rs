use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::path::Path;

use glium::Program;
use glium::backend::Facade;
use glium::backend::glutin_backend::GlutinFacade;
use glium::texture::cubemap::Cubemap;
use glium::texture::{UncompressedFloatFormat, MipmapsOption, Dimensions, RawImage2d};
use gl;
use image;

use {Result, oops};

type RawImage = RawImage2d<'static, u8>;


pub fn load_program(facade: &GlutinFacade,
                    vertex_shader_path: &str,
                    fragment_shader_path: &str)
                    -> Result<Program> {

    let vertex_shader = try!(slurp(&format!("./assets/shaders/{}", vertex_shader_path)));
    let fragment_shader = try!(slurp(&format!("./assets/shaders/{}", fragment_shader_path)));
    Ok(try!(Program::from_source(facade, &vertex_shader, &fragment_shader, None)))
}

fn load_cubemap_faces(texture_src: &str) -> Result<(u32, Vec<RawImage>)> {
    let parts = ["right", "left", "bottom", "top", "back", "front"];
    let mut size = 0;
    parts.iter()
         .map(|part| {
             let path = &format!("./assets/textures/{}/{}.jpg", texture_src, part);
             let cursor = Cursor::new(try!(slurp_bytes(path)));
             let im = try!(image::load(cursor, image::JPEG)).to_rgba();
             let dim = im.dimensions();
             if size == 0 {
                 size = dim.0;
             }
             if size != dim.0 || size != dim.1 {
                 panic!("Bad cubemap texture size: {:?}", dim);
             }

             Ok(RawImage2d::from_raw_rgba_reversed(im.into_raw(), dim))
         })
         .collect::<Result<Vec<_>>>()
         .map(|v| (size, v))
}

pub fn load_cubemap(facade: &GlutinFacade, texture_src: &str) -> Result<Cubemap> {
    info!("Loading cubemap {} ...", texture_src);
    let (size, faces) = try!(load_cubemap_faces(texture_src));

    let result = unsafe {
        let mut id = 0;
        facade.get_context().exec_in_context(|| {
            let window = facade.get_window().expect("can't load cubemap in headless context");
            gl::load_with(|s| window.get_proc_address(s) as *const _);

            id = cubemap_id(faces, size)
        });
        debug!("Cubemap id {}", id);
        Cubemap::from_id(facade,
                         UncompressedFloatFormat::U8U8U8U8,
                         id,
                         true,
                         MipmapsOption::NoMipmap,
                         Dimensions::Cubemap { dimension: size })
    };
    info!("    ...Done!");
    Ok(result)
}

unsafe fn cubemap_id(faces: Vec<RawImage>, size: u32) -> u32 {
    let mut result: u32 = 0;
    gl::GenTextures(1, &mut result);
    gl::BindTexture(gl::TEXTURE_CUBE_MAP, result);
    for (i, face) in faces.iter().enumerate() {
        let bind_point = gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32;
        let size = size as i32;
        gl::TexImage2D(bind_point,
                       0,
                       gl::RGBA as i32,
                       size,
                       size,
                       0,
                       gl::RGBA,
                       gl::UNSIGNED_BYTE,
                       face.data.as_ptr() as *const _);
    }

    gl::TexParameteri(gl::TEXTURE_CUBE_MAP,
                      gl::TEXTURE_MAG_FILTER,
                      gl::LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_CUBE_MAP,
                      gl::TEXTURE_MIN_FILTER,
                      gl::LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_CUBE_MAP,
                      gl::TEXTURE_WRAP_S,
                      gl::CLAMP_TO_EDGE as i32);
    gl::TexParameteri(gl::TEXTURE_CUBE_MAP,
                      gl::TEXTURE_WRAP_T,
                      gl::CLAMP_TO_EDGE as i32);
    gl::TexParameteri(gl::TEXTURE_CUBE_MAP,
                      gl::TEXTURE_WRAP_R,
                      gl::CLAMP_TO_EDGE as i32);
    gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);

    result
}

pub fn slurp<P: AsRef<Path>>(path: P) -> Result<String> {
    let name = path.as_ref().display().to_string();
    let bytes = try!(slurp_bytes(path));

    String::from_utf8(bytes).map_err(|e| oops(format!("invalid utf-8 in {}", name), e))
}


fn slurp_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let name = path.as_ref().display().to_string();
    let mut file = try!(File::open(path).map_err(|e| oops(format!("failed to read {}", name), e)));
    let mut data = vec![];
    try!(file.read_to_end(&mut data).map_err(|e| oops(format!("failed to read {}", name), e)));
    Ok(data)
}

pub fn load_texture<P: AsRef<Path>>(path: P) -> RawImage2d<'static, u8> {
    let bytes = slurp_bytes(path).expect("Failed to load a texture");
    let im = image::load(Cursor::new(bytes), image::PNG)
                 .expect("Failed to load a texture")
                 .to_rgba();
    let dim = im.dimensions();
    RawImage2d::from_raw_rgba_reversed(im.into_raw(), dim)
}

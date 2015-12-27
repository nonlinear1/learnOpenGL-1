#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
}

implement_vertex!(Vertex, position);

impl Vertex {
    pub fn many(verts: Vec<f32>) -> Vec<Vertex> {
        let m = 3;
        if verts.len() % m != 0 {
            panic!("Number of coordinates should be divisible by {}, but it was {}",
                   m,
                   verts.len())
        }
        verts.chunks(m)
             .map(|p| Vertex { position: [p[0], p[1], p[2]] })
             .collect()
    }
}

#[derive(Clone, Copy)]
pub struct VertexNormal {
    position: [f32; 3],
    normal: [f32; 3],
}

implement_vertex!(VertexNormal, position, normal);

impl VertexNormal {
    pub fn many(verts: Vec<f32>) -> Vec<VertexNormal> {
        let m = 6;
        if verts.len() % m != 0 {
            panic!("Number of coordinates should be divisible by {}, but it was {}",
                   m,
                   verts.len())
        }
        verts.chunks(m)
             .map(|p| {
                 VertexNormal {
                     position: [p[0], p[1], p[2]],
                     normal: [p[3], p[4], p[5]],
                 }
             })
             .collect()
    }
}

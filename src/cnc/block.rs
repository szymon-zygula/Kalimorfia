use crate::render::generic_mesh::{ClassicVertex, GlMesh, Mesh, Triangle};
use nalgebra::{point, vector, Vector2, Vector3};

pub struct Block {
    sampling: Vector2<usize>,
    sampling_f32: Vector2<f32>,
    sample_size: Vector2<f32>,
    size: Vector3<f32>,
    heights: Vec<f32>, // Values on the borders serve as sentinels and are always 0
}

impl Block {
    pub fn new(sampling: Vector2<usize>, size: Vector3<f32>) -> Self {
        Self {
            sample_size: vector![size.x / sampling.x as f32, size.y / sampling.y as f32],
            heights: vec![0.0; sampling.x * sampling.y],
            sampling_f32: vector![sampling.x as f32, sampling.y as f32],
            sampling,
            size,
        }
    }

    fn heights_idx(&self, x: usize, y: usize) -> usize {
        x + y * self.sampling.y
    }

    pub fn height(&self, x: usize, y: usize) -> f32 {
        self.heights[self.heights_idx(x, y)]
    }

    pub fn height_mut(&mut self, x: usize, y: usize) -> &mut f32 {
        &mut self.heights[self.heights_idx(x, y)]
    }

    pub fn cut(&mut self, x: usize, y: usize, height: f32) -> bool {
        if self.height(x, y) > height {
            *self.height_mut(x, y) = height;
            true
        } else {
            false
        }
    }

    pub fn generate_mesh(&self, gl: &glow::Context) -> GlMesh {
        let mut vertices = Vec::with_capacity(12 * self.sampling.x * self.sampling.y);
        let mut triangles = Vec::with_capacity(6 * self.sampling.x * self.sampling.y);

        self.mesh_tops(&mut vertices, &mut triangles);
        self.mesh_walls(&mut vertices, &mut triangles);

        let mesh = Mesh {
            vertices,
            triangles,
        };

        GlMesh::new(gl, &mesh)
    }

    fn mesh_tops(&self, vertices: &mut Vec<ClassicVertex>, triangles: &mut Vec<Triangle>) {
        for x in 0..self.sampling.x {
            for y in 0..self.sampling.y {
                vertices.extend_from_slice(&self.sample_top_vertices(x, y));
                triangles.extend_from_slice(&self.sample_top_triangles(x, y, vertices.len()));
            }
        }
    }

    fn sample_top_vertices(&self, x: usize, y: usize) -> [ClassicVertex; 4] {
        let height = self.height(x, y);
        let x = x as f32;
        let y = y as f32;
        let base_point = point![x * self.sample_size.x, y * self.sample_size.y, height];

        // ^y
        // |
        // +-->x
        //
        // 2   3
        //
        // 0   1

        [
            ClassicVertex::new(base_point, vector![0.0, 0.0, 1.0]),
            ClassicVertex::new(
                base_point + vector![self.sample_size.x, 0.0, 0.0],
                vector![0.0, 0.0, 1.0],
            ),
            ClassicVertex::new(
                base_point + vector![0.0, self.sample_size.y, 0.0],
                vector![0.0, 0.0, 1.0],
            ),
            ClassicVertex::new(
                base_point + vector![self.sample_size.x, self.sample_size.y, 0.0],
                vector![0.0, 0.0, 1.0],
            ),
        ]
    }

    // Assume that the last added vertices belong to the same top
    fn sample_top_triangles(&self, x: usize, y: usize, vertices_len: usize) -> [Triangle; 2] {
        let vertices_len = vertices_len as u32;

        [
            Triangle([vertices_len - 3, vertices_len - 2, vertices_len - 1]),
            Triangle([vertices_len, vertices_len - 1, vertices_len - 2]),
        ]
    }

    fn mesh_walls(&self, vertices: &mut Vec<ClassicVertex>, triangles: &mut Vec<Triangle>) {
        self.mesh_x_walls(vertices, triangles);
        self.mesh_y_walls(vertices, triangles);
    }

    // ^y
    // |
    // +-->x
    //
    // -----
    // -----
    // -----
    fn mesh_x_walls(&self, vertices: &mut Vec<ClassicVertex>, triangles: &mut Vec<Triangle>) {
        for y in 1..self.sampling.y {
            self.mesh_x_internal_wall_row(vertices, triangles, y);
        }

        for x in 0..self.sampling.x {
            self.mesh_x_wall(vertices, triangles, x, 0, self.height(x, 0), 0.0);
        }

        for x in 0..self.sampling.x {
            self.mesh_x_wall(
                vertices,
                triangles,
                x,
                self.sampling.y,
                0.0,
                self.height(x, self.sampling.y - 1),
            );
        }
    }

    fn mesh_x_internal_wall_row(
        &self,
        vertices: &mut Vec<ClassicVertex>,
        triangles: &mut Vec<Triangle>,
        y: usize,
    ) {
        for x in 0..self.sampling.x {
            self.mesh_x_wall(
                vertices,
                triangles,
                x,
                y,
                self.height(x, y),
                self.height(x, y - 1),
            );
        }
    }

    fn mesh_x_wall(
        &self,
        vertices: &mut Vec<ClassicVertex>,
        triangles: &mut Vec<Triangle>,
        x: usize,
        y: usize,
        my_height: f32,
        neighbor_height: f32,
    ) {
        let height_difference = my_height - neighbor_height;
        //
        // ^z
        // |
        // +-->x
        //
        // 2   3
        //
        // 0   1
        let normal = if height_difference > 0.0 {
            vector![0.0, -1.0, 0.0]
        } else {
            vector![0.0, 1.0, 0.0]
        };

        let x = x as f32;
        let y = y as f32;

        let base_point = point![x * self.sample_size.x, y * self.sample_size.y, 0.0];

        vertices.push(ClassicVertex::new(
            base_point + vector![0.0, 0.0, neighbor_height],
            normal,
        ));
        vertices.push(ClassicVertex::new(
            base_point + vector![self.sample_size.x, 0.0, neighbor_height],
            normal,
        ));
        vertices.push(ClassicVertex::new(
            base_point + vector![0.0, 0.0, my_height],
            normal,
        ));
        vertices.push(ClassicVertex::new(
            base_point + vector![self.sample_size.x, 0.0, my_height],
            normal,
        ));

        let len = vertices.len() as u32;

        triangles.push(Triangle([len - 3, len - 2, len - 1]));
        triangles.push(Triangle([len - 2, len - 1, len]));
    }

    // ^y
    // |
    // +-->x
    //
    // |||||
    // |||||
    // |||||
    fn mesh_y_walls(&self, vertices: &mut Vec<ClassicVertex>, triangles: &mut Vec<Triangle>) {}

    fn mesh_y_internal_wall_row(
        &self,
        vertices: &mut Vec<ClassicVertex>,
        triangles: &mut Vec<Triangle>,
        y: usize,
    ) {
    }
}

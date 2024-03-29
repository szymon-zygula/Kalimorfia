use crate::render::generic_mesh::{CNCBlockVertex, Mesh, Triangle};
use nalgebra::{point, vector, Vector2, Vector3};

#[derive(Clone)]
pub struct Block {
    sampling: Vector2<usize>,
    sample_size: Vector2<f32>,
    heights: Vec<f32>,
    height: f32,
    size: Vector2<f32>,
    pub base_height: f32,
}

impl Block {
    pub fn new(sampling: Vector2<usize>, size: Vector3<f32>) -> Self {
        Self {
            sample_size: vector![size.x / sampling.x as f32, size.y / sampling.y as f32],
            heights: vec![size.z; sampling.x * sampling.y],
            sampling,
            height: size.z,
            size: vector![size.x, size.y],
            base_height: size.z / 10.0,
        }
    }

    pub fn sample_size(&self) -> &Vector2<f32> {
        &self.sample_size
    }

    fn heights_idx(&self, x: usize, y: usize) -> usize {
        x + y * self.sampling.x
    }

    pub fn block_height(&self) -> f32 {
        self.height
    }

    pub fn height(&self, x: usize, y: usize) -> f32 {
        self.heights[self.heights_idx(x, y)]
    }

    pub fn height_mut(&mut self, x: usize, y: usize) -> &mut f32 {
        let idx = self.heights_idx(x, y);
        &mut self.heights[idx]
    }

    pub fn cut(&mut self, x: usize, y: usize, height: f32) -> bool {
        if self.height(x, y) > height {
            *self.height_mut(x, y) = height;
            true
        } else {
            false
        }
    }

    pub fn generate_mesh(&self) -> Mesh<CNCBlockVertex> {
        let mut vertices = Vec::with_capacity(12 * self.sampling.x * self.sampling.y);
        let mut triangles = Vec::with_capacity(6 * self.sampling.x * self.sampling.y);

        self.mesh_tops(&mut vertices, &mut triangles);
        self.mesh_walls(&mut vertices, &mut triangles);

        Mesh {
            vertices,
            triangles,
        }
    }

    fn mesh_tops(&self, vertices: &mut Vec<CNCBlockVertex>, triangles: &mut Vec<Triangle>) {
        for x in 0..self.sampling.x {
            for y in 0..self.sampling.y {
                vertices.extend_from_slice(&self.sample_top_vertices(x, y));
                triangles.extend_from_slice(&self.sample_top_triangles(x, y));
            }
        }
    }

    fn sample_top_vertices(&self, x: usize, y: usize) -> [CNCBlockVertex; 4] {
        let height = 0.0; // self.height(x, y);
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
            CNCBlockVertex::new(base_point, vector![0.0, 0.0, 1.0], x, y),
            CNCBlockVertex::new(
                base_point + vector![self.sample_size.x, 0.0, 0.0],
                vector![0.0, 0.0, 1.0],
                x,
                y,
            ),
            CNCBlockVertex::new(
                base_point + vector![0.0, self.sample_size.y, 0.0],
                vector![0.0, 0.0, 1.0],
                x,
                y,
            ),
            CNCBlockVertex::new(
                base_point + vector![self.sample_size.x, self.sample_size.y, 0.0],
                vector![0.0, 0.0, 1.0],
                x,
                y,
            ),
        ]
    }

    fn sample_top_triangles(&self, x: usize, y: usize) -> [Triangle; 2] {
        let vertices_offset = 4 * (y + self.sampling.y * x) as u32;

        [
            Triangle([vertices_offset, vertices_offset + 1, vertices_offset + 2]),
            Triangle([
                vertices_offset + 3,
                vertices_offset + 2,
                vertices_offset + 1,
            ]),
        ]
    }

    fn mesh_walls(&self, vertices: &mut Vec<CNCBlockVertex>, triangles: &mut Vec<Triangle>) {
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
    fn mesh_x_walls(&self, vertices: &mut Vec<CNCBlockVertex>, triangles: &mut Vec<Triangle>) {
        for y in 1..self.sampling.y {
            self.mesh_x_internal_wall_row(vertices, triangles, y);
        }

        for x in 0..self.sampling.x {
            self.mesh_x_wall(
                vertices, triangles, x, 0, 0.0, // self.height(x, 0),
                0.0,
            );
        }

        for x in 0..self.sampling.x {
            self.mesh_x_wall(
                vertices,
                triangles,
                x,
                self.sampling.y,
                0.0,
                0.0, // self.height(x, self.sampling.y - 1),
            );
        }
    }

    fn mesh_x_internal_wall_row(
        &self,
        vertices: &mut Vec<CNCBlockVertex>,
        triangles: &mut Vec<Triangle>,
        y: usize,
    ) {
        for x in 0..self.sampling.x {
            self.mesh_x_wall(
                vertices, triangles, x, y,
                // self.height(x, y),
                // self.height(x, y - 1),
                0.0, 0.0,
            );
        }
    }

    fn mesh_x_wall(
        &self,
        vertices: &mut Vec<CNCBlockVertex>,
        triangles: &mut Vec<Triangle>,
        x: usize,
        y: usize,
        my_height: f32,
        neighbor_height: f32,
    ) {
        let height_difference = my_height - neighbor_height;

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

        vertices.push(CNCBlockVertex::new(
            base_point + vector![0.0, 0.0, neighbor_height],
            normal,
            x,
            y - 1.0,
        ));
        vertices.push(CNCBlockVertex::new(
            base_point + vector![self.sample_size.x, 0.0, neighbor_height],
            normal,
            x,
            y - 1.0,
        ));
        vertices.push(CNCBlockVertex::new(
            base_point + vector![0.0, 0.0, my_height],
            normal,
            x,
            y,
        ));
        vertices.push(CNCBlockVertex::new(
            base_point + vector![self.sample_size.x, 0.0, my_height],
            normal,
            x,
            y,
        ));

        let len = vertices.len() as u32;

        triangles.push(Triangle([len - 4, len - 3, len - 2]));
        triangles.push(Triangle([len - 3, len - 2, len - 1]));
    }

    // ^y
    // |
    // +-->x
    //
    // |||||
    // |||||
    // |||||
    fn mesh_y_walls(&self, vertices: &mut Vec<CNCBlockVertex>, triangles: &mut Vec<Triangle>) {
        for x in 1..self.sampling.x {
            self.mesh_y_internal_wall_row(vertices, triangles, x);
        }

        for y in 0..self.sampling.y {
            self.mesh_y_wall(
                vertices, triangles, 0, y, // self.height(0, y),
                0.0, 0.0,
            );
        }

        for y in 0..self.sampling.y {
            self.mesh_y_wall(
                vertices,
                triangles,
                self.sampling.x,
                y,
                0.0,
                0.0, // self.height(self.sampling.x - 1, y),
            );
        }
    }

    fn mesh_y_internal_wall_row(
        &self,
        vertices: &mut Vec<CNCBlockVertex>,
        triangles: &mut Vec<Triangle>,
        x: usize,
    ) {
        for y in 0..self.sampling.y {
            self.mesh_y_wall(
                vertices, triangles, x, y, 0.0,
                0.0, // self.height(x, y),
                    // self.height(x - 1, y),
            );
        }
    }

    fn mesh_y_wall(
        &self,
        vertices: &mut Vec<CNCBlockVertex>,
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
        // +-->y
        //
        // 2   3
        //
        // 0   1
        let normal = if height_difference > 0.0 {
            vector![-1.0, 0.0, 0.0]
        } else {
            vector![1.0, 0.0, 0.0]
        };

        let x = x as f32;
        let y = y as f32;

        let base_point = point![x * self.sample_size.x, y * self.sample_size.y, 0.0];

        vertices.push(CNCBlockVertex::new(
            base_point + vector![0.0, 0.0, neighbor_height],
            normal,
            x - 1.0,
            y,
        ));
        vertices.push(CNCBlockVertex::new(
            base_point + vector![0.0, self.sample_size.y, neighbor_height],
            normal,
            x - 1.0,
            y,
        ));
        vertices.push(CNCBlockVertex::new(
            base_point + vector![0.0, 0.0, my_height],
            normal,
            x,
            y,
        ));
        vertices.push(CNCBlockVertex::new(
            base_point + vector![0.0, self.sample_size.y, my_height],
            normal,
            x,
            y,
        ));

        let len = vertices.len() as u32;

        triangles.push(Triangle([len - 4, len - 3, len - 2]));
        triangles.push(Triangle([len - 3, len - 2, len - 1]));
    }

    pub fn sampling(&self) -> &Vector2<usize> {
        &self.sampling
    }

    pub fn size(&self) -> &Vector2<f32> {
        &self.size
    }

    pub fn mill_to_block(&self, position: &Vector2<f32>) -> Vector2<i32> {
        vector![
            ((position.x + 0.5 * self.size.x) / self.sample_size.x).floor() as i32,
            ((position.y + 0.5 * self.size.y) / self.sample_size.y).floor() as i32
        ]
    }

    pub fn contains(&self, loc: &Vector2<i32>) -> bool {
        loc.x >= 0 && loc.y >= 0 && loc.x < self.sampling.x as i32 && loc.y < self.sampling.y as i32
    }

    pub fn raw_heights(&self) -> &Vec<f32> {
        &self.heights
    }
}

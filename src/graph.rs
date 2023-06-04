use itertools::Itertools;

use crate::entities::entity::EntityCollection;

pub struct C0EdgeGraph {
    edges: Vec<C0Edge>,
}

impl C0EdgeGraph {
    pub fn new(entities: &EntityCollection, selected: &[usize]) -> Self {
        Self {
            edges: selected
                .iter()
                .flat_map(|id| {
                    entities
                        .get(id)
                        .unwrap()
                        .borrow()
                        .as_c0_surface()
                        .unwrap()
                        .patch_edges()
                })
                .collect(),
        }
    }

    pub fn vertices(&self) -> Vec<usize> {
        self.edges
            .iter()
            .map(|e| e.endpoints())
            .flat_map(|(v1, v2)| [v1, v2])
            .unique()
            .collect()
    }

    pub fn edges(&self) -> &[C0Edge] {
        &self.edges
    }

    pub fn oriented_edge(&self, vertex1: usize, vertex2: usize) -> Option<C0Edge> {
        self.edges.iter().find_map(|e| {
            let endpoints = e.endpoints();

            (endpoints.0 == vertex1 && endpoints.1 == vertex2)
                .then_some(e.clone())
                .or_else(|| {
                    (endpoints.0 == vertex2 && endpoints.1 == vertex1).then_some(e.reverese())
                })
        })
    }

    pub fn find_triangles(&self) -> Vec<C0EdgeTriangle> {
        self.edges()
            .iter()
            .cartesian_product(self.vertices().iter().copied())
            .map(|(edge0, vertex2)| (edge0, vertex2, edge0.endpoints()))
            .filter_map(|(edge0, vertex2, (vertex0, vertex1))| {
                (vertex0 < vertex2 && vertex1 < vertex2)
                    .then_some(
                        self.oriented_edge(vertex1, vertex2)
                            .zip(self.oriented_edge(vertex2, vertex0))
                            .map(|(edge1, edge2)| C0EdgeTriangle([edge0.clone(), edge1, edge2])),
                    )
                    .flatten()
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct C0EdgeTriangle([C0Edge; 3]);

#[derive(Clone, Debug)]
pub struct C0Edge {
    points: [[usize; 4]; 4],
}

impl C0Edge {
    pub fn new(points: [[usize; 4]; 4]) -> Self {
        Self { points }
    }

    pub fn reverese(&self) -> Self {
        Self {
            points: [
                [
                    self.points[0][3],
                    self.points[0][2],
                    self.points[0][1],
                    self.points[0][0],
                ],
                [
                    self.points[1][3],
                    self.points[1][2],
                    self.points[1][1],
                    self.points[1][0],
                ],
                [
                    self.points[2][3],
                    self.points[2][2],
                    self.points[2][1],
                    self.points[2][0],
                ],
                [
                    self.points[3][3],
                    self.points[3][2],
                    self.points[3][1],
                    self.points[3][0],
                ],
            ],
        }
    }

    pub fn edge_points(&self) -> &[usize; 4] {
        &self.points[0]
    }

    pub fn is_endpoint(&self, id: usize) -> bool {
        let endpoints = self.endpoints();
        endpoints.0 == id || endpoints.1 == id
    }

    pub fn endpoints(&self) -> (usize, usize) {
        let edge_points = self.edge_points();
        (edge_points[0], edge_points[3])
    }
}

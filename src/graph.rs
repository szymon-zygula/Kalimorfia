use itertools::Itertools;

use crate::entities::entity::EntityCollection;

#[derive(Clone)]
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
                .filter(|e| {
                    // Filter degenerate loop-edges
                    let (a, b) = e.endpoints();
                    a != b
                })
                .map(|e| {
                    let endpoints = e.endpoints();

                    if endpoints.0 > endpoints.1 {
                        e.reverse()
                    } else {
                        e
                    }
                })
                .sorted_by(|e1, e2| e1.edge_points().cmp(e2.edge_points()))
                .group_by(|e| *e.edge_points())
                .into_iter()
                .filter_map(|(_, val)| {
                    let edges: Vec<_> = val.collect();
                    (edges.len() == 1).then_some(edges[0].clone())
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

    pub fn oriented_edges(&self, vertex0: usize, vertex1: usize) -> Vec<C0Edge> {
        self.edges
            .iter()
            .filter_map(|e| {
                let endpoints = e.endpoints();

                (endpoints.0 == vertex0 && endpoints.1 == vertex1)
                    .then_some(e.clone())
                    .or_else(|| {
                        (endpoints.0 == vertex1 && endpoints.1 == vertex0).then_some(e.reverse())
                    })
            })
            .collect()
    }

    pub fn find_triangles(&self) -> Vec<C0EdgeTriangle> {
        self.edges()
            .iter()
            .cartesian_product(self.vertices().iter().copied())
            .map(|(edge0, vertex2)| (edge0, vertex2, edge0.endpoints()))
            .flat_map(|(edge0, vertex2, (vertex0, vertex1))| {
                if vertex0 < vertex2 && vertex1 < vertex2 {
                    self.oriented_edges(vertex1, vertex2)
                        .iter()
                        .cartesian_product(self.oriented_edges(vertex2, vertex0))
                        .map(|(edge1, edge2)| C0EdgeTriangle([edge0.clone(), edge1.clone(), edge2]))
                        .collect::<Vec<C0EdgeTriangle>>()
                } else {
                    Vec::new()
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct C0EdgeTriangle(pub [C0Edge; 3]);

#[derive(Clone, Debug)]
pub struct C0Edge {
    pub points: [[usize; 4]; 4],
}

impl C0Edge {
    pub fn new(points: [[usize; 4]; 4]) -> Self {
        Self { points }
    }

    pub fn reverse(&self) -> Self {
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

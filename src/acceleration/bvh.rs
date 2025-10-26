//! Bounding Volume Hierarchy (BVH) for triangle acceleration

use crate::core::tuples::{Point, Tuple};
use crate::geometry::triangles::SmoothTriangle as SmoothTriangleObject;
use crate::geometry::triangles::Triangle as TriangleObject;

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Point,
    pub max: Point,
}

impl AABB {
    pub fn new(min: Point, max: Point) -> Self {
        Self { min, max }
    }

    pub fn empty() -> Self {
        Self {
            min: Point::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            max: Point::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }

    pub fn expand_by_point(&mut self, point: Point) {
        self.min = Point::new(
            self.min.x().min(point.x()),
            self.min.y().min(point.y()),
            self.min.z().min(point.z()),
        );
        self.max = Point::new(
            self.max.x().max(point.x()),
            self.max.y().max(point.y()),
            self.max.z().max(point.z()),
        );
    }

    pub fn expand_by_aabb(&mut self, other: &AABB) {
        self.expand_by_point(other.min);
        self.expand_by_point(other.max);
    }

    pub fn surface_area(&self) -> f64 {
        let dx = self.max.x() - self.min.x();
        let dy = self.max.y() - self.min.y();
        let dz = self.max.z() - self.min.z();
        2.0 * (dx * dy + dy * dz + dz * dx)
    }

    pub fn centroid(&self) -> Point {
        Point::new(
            (self.min.x() + self.max.x()) * 0.5,
            (self.min.y() + self.max.y()) * 0.5,
            (self.min.z() + self.max.z()) * 0.5,
        )
    }
}

#[derive(Debug, Clone)]
pub struct BVHTriangle {
    pub index: usize,
    pub bounds: AABB,
    pub centroid: Point,
}

impl BVHTriangle {
    pub fn from_triangle(triangle: &TriangleObject, index: usize) -> Self {
        let mut bounds = AABB::empty();
        bounds.expand_by_point(triangle.p1);
        bounds.expand_by_point(triangle.p2);
        bounds.expand_by_point(triangle.p3);
        let centroid = bounds.centroid();

        Self {
            index,
            bounds,
            centroid,
        }
    }

    pub fn from_smooth_triangle(triangle: &SmoothTriangleObject, index: usize) -> Self {
        let mut bounds = AABB::empty();
        bounds.expand_by_point(triangle.p1);
        bounds.expand_by_point(triangle.p2);
        bounds.expand_by_point(triangle.p3);
        let centroid = bounds.centroid();

        Self {
            index,
            bounds,
            centroid,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BVHNode {
    /// Internal node with left and right children
    Internal {
        bounds: AABB,
        left: Box<BVHNode>,
        right: Box<BVHNode>,
    },
    /// Leaf node containing triangle indices
    Leaf { bounds: AABB, triangles: Vec<usize> },
}

impl BVHNode {
    pub fn bounds(&self) -> &AABB {
        match self {
            BVHNode::Internal { bounds, .. } => bounds,
            BVHNode::Leaf { bounds, .. } => bounds,
        }
    }

    pub fn build(triangles: &mut [BVHTriangle], max_leaf_size: usize) -> Self {
        if triangles.len() <= max_leaf_size {
            let mut bounds = AABB::empty();
            for tri in triangles.iter() {
                bounds.expand_by_aabb(&tri.bounds);
            }
            let indices = triangles.iter().map(|t| t.index).collect();
            return BVHNode::Leaf {
                bounds,
                triangles: indices,
            };
        }

        let mut centroid_bounds = AABB::empty();
        for tri in triangles.iter() {
            centroid_bounds.expand_by_point(tri.centroid);
        }

        let extent = Point::new(
            centroid_bounds.max.x() - centroid_bounds.min.x(),
            centroid_bounds.max.y() - centroid_bounds.min.y(),
            centroid_bounds.max.z() - centroid_bounds.min.z(),
        );

        let axis = if extent.x() > extent.y() && extent.x() > extent.z() {
            0
        } else if extent.y() > extent.z() {
            1
        } else {
            2
        };

        triangles.sort_by(|a, b| {
            let a_val = match axis {
                0 => a.centroid.x(),
                1 => a.centroid.y(),
                _ => a.centroid.z(),
            };
            let b_val = match axis {
                0 => b.centroid.x(),
                1 => b.centroid.y(),
                _ => b.centroid.z(),
            };
            a_val.partial_cmp(&b_val).unwrap()
        });

        let mid = triangles.len() / 2;
        let (left_triangles, right_triangles) = triangles.split_at_mut(mid);

        let left = Box::new(Self::build(left_triangles, max_leaf_size));
        let right = Box::new(Self::build(right_triangles, max_leaf_size));

        let mut bounds = AABB::empty();
        bounds.expand_by_aabb(left.bounds());
        bounds.expand_by_aabb(right.bounds());

        BVHNode::Internal {
            bounds,
            left,
            right,
        }
    }
}

/// Flattened BVH node for GPU (no pointers, just indices)
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "gpu", derive(bytemuck::Pod, bytemuck::Zeroable))]
pub struct GpuBVHNode {
    pub min: [f32; 3],
    pub _min_padding: f32,
    pub max: [f32; 3],
    pub _max_padding: f32,
    /// For internal nodes: index of left child
    /// For leaf nodes: index of first triangle in triangle index buffer
    pub left_or_first: u32,
    /// For internal nodes: index of right child (left + 1 typically)
    /// For leaf nodes: number of triangles in this leaf
    pub right_or_count: u32,
    /// 1 = leaf node, 0 = internal node
    pub is_leaf: u32,
    pub _padding: u32,
}

/// Bounding Volume Hierarchy for triangle acceleration
pub struct BVH {
    /// Root node of the BVH tree
    pub root: BVHNode,
    /// Flattened nodes for GPU upload
    pub flat_nodes: Vec<GpuBVHNode>,
    /// Triangle indices for leaf nodes
    pub triangle_indices: Vec<u32>,
}

impl BVH {
    pub fn build(mut triangles: Vec<BVHTriangle>) -> Self {
        let root = BVHNode::build(&mut triangles, 1); // Max 1 triangle per leaf for maximum precision

        let mut flat_nodes = Vec::new();
        let mut triangle_indices = Vec::new();

        Self::flatten_tree(&root, &mut flat_nodes, &mut triangle_indices);

        Self {
            root,
            flat_nodes,
            triangle_indices,
        }
    }

    fn flatten_tree(
        node: &BVHNode,
        flat_nodes: &mut Vec<GpuBVHNode>,
        triangle_indices: &mut Vec<u32>,
    ) -> u32 {
        let my_index = flat_nodes.len() as u32;

        match node {
            BVHNode::Leaf { bounds, triangles } => {
                let first_tri_idx = triangle_indices.len() as u32;
                for &tri_idx in triangles.iter() {
                    triangle_indices.push(tri_idx as u32);
                }

                flat_nodes.push(GpuBVHNode {
                    min: [
                        bounds.min.x() as f32,
                        bounds.min.y() as f32,
                        bounds.min.z() as f32,
                    ],
                    _min_padding: 0.0,
                    max: [
                        bounds.max.x() as f32,
                        bounds.max.y() as f32,
                        bounds.max.z() as f32,
                    ],
                    _max_padding: 0.0,
                    left_or_first: first_tri_idx,
                    right_or_count: triangles.len() as u32,
                    is_leaf: 1,
                    _padding: 0,
                });
            }
            BVHNode::Internal {
                bounds,
                left,
                right,
            } => {
                // Reserve space for this node
                flat_nodes.push(GpuBVHNode {
                    min: [0.0; 3],
                    _min_padding: 0.0,
                    max: [0.0; 3],
                    _max_padding: 0.0,
                    left_or_first: 0,
                    right_or_count: 0,
                    is_leaf: 0,
                    _padding: 0,
                });

                // Flatten left child (always immediately after parent)
                let left_idx = Self::flatten_tree(left, flat_nodes, triangle_indices);

                // Flatten right child
                let right_idx = Self::flatten_tree(right, flat_nodes, triangle_indices);

                // Update this node with correct data
                flat_nodes[my_index as usize] = GpuBVHNode {
                    min: [
                        bounds.min.x() as f32,
                        bounds.min.y() as f32,
                        bounds.min.z() as f32,
                    ],
                    _min_padding: 0.0,
                    max: [
                        bounds.max.x() as f32,
                        bounds.max.y() as f32,
                        bounds.max.z() as f32,
                    ],
                    _max_padding: 0.0,
                    left_or_first: left_idx,
                    right_or_count: right_idx,
                    is_leaf: 0,
                    _padding: 0,
                };
            }
        }

        my_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_empty_has_inverted_bounds() {
        let aabb = AABB::empty();
        assert!(aabb.min.x() > aabb.max.x());
        assert!(aabb.min.y() > aabb.max.y());
        assert!(aabb.min.z() > aabb.max.z());
    }

    #[test]
    fn test_aabb_expand_by_point() {
        let mut aabb = AABB::empty();
        aabb.expand_by_point(Point::new(1.0, 2.0, 3.0));

        assert_eq!(aabb.min.x(), 1.0);
        assert_eq!(aabb.min.y(), 2.0);
        assert_eq!(aabb.min.z(), 3.0);
        assert_eq!(aabb.max.x(), 1.0);
        assert_eq!(aabb.max.y(), 2.0);
        assert_eq!(aabb.max.z(), 3.0);

        aabb.expand_by_point(Point::new(-1.0, 5.0, 1.0));
        assert_eq!(aabb.min.x(), -1.0);
        assert_eq!(aabb.min.y(), 2.0);
        assert_eq!(aabb.min.z(), 1.0);
        assert_eq!(aabb.max.x(), 1.0);
        assert_eq!(aabb.max.y(), 5.0);
        assert_eq!(aabb.max.z(), 3.0);
    }

    #[test]
    fn test_aabb_expand_by_aabb() {
        let mut aabb1 = AABB::new(Point::new(0.0, 0.0, 0.0), Point::new(1.0, 1.0, 1.0));

        let aabb2 = AABB::new(Point::new(0.5, 0.5, 0.5), Point::new(2.0, 2.0, 2.0));

        aabb1.expand_by_aabb(&aabb2);

        assert_eq!(aabb1.min.x(), 0.0);
        assert_eq!(aabb1.min.y(), 0.0);
        assert_eq!(aabb1.min.z(), 0.0);
        assert_eq!(aabb1.max.x(), 2.0);
        assert_eq!(aabb1.max.y(), 2.0);
        assert_eq!(aabb1.max.z(), 2.0);
    }

    #[test]
    fn test_aabb_surface_area() {
        let aabb = AABB::new(Point::new(0.0, 0.0, 0.0), Point::new(2.0, 3.0, 4.0));

        let area = aabb.surface_area();
        assert_eq!(area, 2.0 * (2.0 * 3.0 + 3.0 * 4.0 + 4.0 * 2.0));
        assert_eq!(area, 52.0);
    }

    #[test]
    fn test_aabb_centroid() {
        let aabb = AABB::new(Point::new(0.0, 0.0, 0.0), Point::new(4.0, 6.0, 8.0));

        let centroid = aabb.centroid();
        assert_eq!(centroid.x(), 2.0);
        assert_eq!(centroid.y(), 3.0);
        assert_eq!(centroid.z(), 4.0);
    }

    #[test]
    fn test_bvh_triangle_from_triangle() {
        use crate::geometry::triangles::Triangle;

        let tri = Triangle::new(
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
            Point::new(0.0, 1.0, 0.0),
        );

        let bvh_tri = BVHTriangle::from_triangle(&tri, 42);

        assert_eq!(bvh_tri.index, 42);
        assert_eq!(bvh_tri.bounds.min.x(), 0.0);
        assert_eq!(bvh_tri.bounds.min.y(), 0.0);
        assert_eq!(bvh_tri.bounds.min.z(), 0.0);
        assert_eq!(bvh_tri.bounds.max.x(), 1.0);
        assert_eq!(bvh_tri.bounds.max.y(), 1.0);
        assert_eq!(bvh_tri.bounds.max.z(), 0.0);
    }

    #[test]
    fn test_bvh_node_build_single_triangle() {
        use crate::geometry::triangles::Triangle;

        let tri = Triangle::new(
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
            Point::new(0.0, 1.0, 0.0),
        );

        let mut bvh_triangles = vec![BVHTriangle::from_triangle(&tri, 0)];
        let node = BVHNode::build(&mut bvh_triangles, 1);

        match node {
            BVHNode::Leaf { triangles, .. } => {
                assert_eq!(triangles.len(), 1);
                assert_eq!(triangles[0], 0);
            }
            BVHNode::Internal { .. } => panic!("Expected leaf node for single triangle"),
        }
    }

    #[test]
    fn test_bvh_node_build_multiple_triangles() {
        use crate::geometry::triangles::Triangle;

        let tri1 = Triangle::new(
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
            Point::new(0.0, 1.0, 0.0),
        );
        let tri2 = Triangle::new(
            Point::new(10.0, 0.0, 0.0),
            Point::new(11.0, 0.0, 0.0),
            Point::new(10.0, 1.0, 0.0),
        );

        let mut bvh_triangles = vec![
            BVHTriangle::from_triangle(&tri1, 0),
            BVHTriangle::from_triangle(&tri2, 1),
        ];

        let node = BVHNode::build(&mut bvh_triangles, 1);

        match node {
            BVHNode::Internal { left, right, .. } => {
                assert!(matches!(left.as_ref(), BVHNode::Leaf { .. }));
                assert!(matches!(right.as_ref(), BVHNode::Leaf { .. }));
            }
            BVHNode::Leaf { .. } => {
                panic!("Expected internal node for multiple triangles with max_leaf_size=1")
            }
        }
    }

    #[test]
    fn test_bvh_flattening_produces_valid_indices() {
        use crate::geometry::triangles::Triangle;

        let tri1 = Triangle::new(
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
            Point::new(0.0, 1.0, 0.0),
        );
        let tri2 = Triangle::new(
            Point::new(10.0, 0.0, 0.0),
            Point::new(11.0, 0.0, 0.0),
            Point::new(10.0, 1.0, 0.0),
        );

        let bvh_triangles = vec![
            BVHTriangle::from_triangle(&tri1, 0),
            BVHTriangle::from_triangle(&tri2, 1),
        ];

        let bvh = BVH::build(bvh_triangles);

        assert!(!bvh.flat_nodes.is_empty());
        assert_eq!(bvh.triangle_indices.len(), 2);

        for node in &bvh.flat_nodes {
            if node.is_leaf == 1 {
                let first_idx = node.left_or_first as usize;
                let count = node.right_or_count as usize;
                assert!(first_idx + count <= bvh.triangle_indices.len());
            } else {
                let left_idx = node.left_or_first as usize;
                let right_idx = node.right_or_count as usize;
                assert!(left_idx < bvh.flat_nodes.len());
                assert!(right_idx < bvh.flat_nodes.len());
            }
        }
    }

    #[test]
    fn test_bvh_preserves_triangle_count() {
        use crate::geometry::triangles::Triangle;

        let triangles: Vec<_> = (0..10)
            .map(|i| {
                Triangle::new(
                    Point::new(i as f64, 0.0, 0.0),
                    Point::new(i as f64 + 1.0, 0.0, 0.0),
                    Point::new(i as f64, 1.0, 0.0),
                )
            })
            .collect();

        let bvh_triangles: Vec<_> = triangles
            .iter()
            .enumerate()
            .map(|(i, tri)| BVHTriangle::from_triangle(tri, i))
            .collect();

        let bvh = BVH::build(bvh_triangles);

        assert_eq!(bvh.triangle_indices.len(), 10);
    }
}

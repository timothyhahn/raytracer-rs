use crate::core::tuples::{Point, Tuple, Vector};
use crate::geometry::groups::Group;
use crate::geometry::triangles::{SmoothTriangle, Triangle};
use crate::rendering::objects::Object;
use std::collections::HashMap;

/// OBJ file parser for Wavefront OBJ format
#[derive(Debug)]
pub struct ObjParser {
    /// Vertices parsed from the file (1-indexed to match OBJ format)
    /// Index 0 is unused, vertices start at index 1
    pub vertices: Vec<Point>,
    /// Vertex normals parsed from the file (1-indexed to match OBJ format)
    /// Index 0 is unused, normals start at index 1
    pub normals: Vec<Vector>,
    /// Default group for ungrouped faces
    pub default_group: Group,
    /// Named groups
    named_groups: HashMap<String, Group>,
    /// Current group name (empty string for default group)
    current_group: String,
    /// Number of lines ignored during parsing
    pub ignored_lines: usize,
}

impl ObjParser {
    pub fn new() -> Self {
        Self {
            vertices: vec![Point::zero()], // Index 0 is unused
            normals: vec![Vector::zero()], // Index 0 is unused
            default_group: Group::new(),
            named_groups: HashMap::new(),
            current_group: String::new(),
            ignored_lines: 0,
        }
    }

    /// Get a named group by name
    pub fn get_group(&self, name: &str) -> Option<&Group> {
        self.named_groups.get(name)
    }

    /// Get a mutable reference to a named group by name
    pub fn get_group_mut(&mut self, name: &str) -> Option<&mut Group> {
        self.named_groups.get_mut(name)
    }

    /// First pass: parse vertices and normals only
    fn parse_line_first_pass(&mut self, line: &str) {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            return;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "v" => self.parse_vertex(&parts[1..]),
            "vn" => self.parse_vertex_normal(&parts[1..]),
            "g" => self.parse_group(&parts[1..]),
            "f" => {} // Skip faces in first pass
            _ => {
                // Ignore unrecognized lines
                self.ignored_lines += 1;
            }
        }
    }

    /// Second pass: parse faces only (after vertices and normals are loaded)
    fn parse_line_second_pass(&mut self, line: &str) {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            return;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "f" => self.parse_face(&parts[1..]),
            "g" => self.parse_group(&parts[1..]),
            _ => {} // Already processed or ignored in first pass
        }
    }

    /// Parse a vertex line (v x y z)
    fn parse_vertex(&mut self, parts: &[&str]) {
        if parts.len() >= 3 {
            if let (Ok(x), Ok(y), Ok(z)) = (
                parts[0].parse::<f64>(),
                parts[1].parse::<f64>(),
                parts[2].parse::<f64>(),
            ) {
                self.vertices.push(Point::new(x, y, z));
            }
        }
    }

    /// Parse a vertex normal line (vn x y z)
    fn parse_vertex_normal(&mut self, parts: &[&str]) {
        if parts.len() >= 3 {
            if let (Ok(x), Ok(y), Ok(z)) = (
                parts[0].parse::<f64>(),
                parts[1].parse::<f64>(),
                parts[2].parse::<f64>(),
            ) {
                self.normals.push(Vector::new(x, y, z));
            }
        }
    }

    /// Parse a face line (f v1 v2 v3 ... or f v1//n1 v2//n2 v3//n3 ... or f v1/t1/n1 ...)
    /// Supports triangles and convex polygons (which are triangulated)
    fn parse_face(&mut self, parts: &[&str]) {
        if parts.len() < 3 {
            return;
        }

        // Parse vertex and normal indices (OBJ uses 1-based indexing)
        // Format: v, v/vt, v//vn, or v/vt/vn
        let mut vertex_indices: Vec<usize> = Vec::new();
        let mut normal_indices: Vec<Option<usize>> = Vec::new();

        for part in parts {
            let components: Vec<&str> = part.split('/').collect();

            // First component is always the vertex index
            if let Ok(v_index) = components[0].parse::<usize>() {
                // Validate vertex index is in bounds
                if v_index == 0 || v_index >= self.vertices.len() {
                    // Invalid vertex index, skip this entire face
                    return;
                }
                vertex_indices.push(v_index);

                // Third component (if present) is the normal index
                if components.len() >= 3 && !components[2].is_empty() {
                    if let Ok(n_index) = components[2].parse::<usize>() {
                        // Validate normal index is in bounds
                        if n_index > 0 && n_index < self.normals.len() {
                            normal_indices.push(Some(n_index));
                        } else {
                            // Invalid normal index, skip this entire face
                            return;
                        }
                    } else {
                        normal_indices.push(None);
                    }
                } else {
                    normal_indices.push(None);
                }
            }
        }

        if vertex_indices.len() < 3 {
            return;
        }

        // Check if all vertices have normals
        let has_normals = normal_indices.iter().all(|n| n.is_some());

        // Fan triangulation: create triangles using vertex 1 as the pivot
        if has_normals {
            let smooth_triangles = self.fan_triangulation_smooth(&vertex_indices, &normal_indices);
            for triangle in smooth_triangles {
                if self.current_group.is_empty() {
                    self.default_group.add_child(
                        Object::SmoothTriangle(triangle),
                        crate::core::matrices::Matrix4::identity(),
                    );
                } else {
                    let group = self
                        .named_groups
                        .entry(self.current_group.clone())
                        .or_default();
                    group.add_child(
                        Object::SmoothTriangle(triangle),
                        crate::core::matrices::Matrix4::identity(),
                    );
                }
            }
        } else {
            let triangles = self.fan_triangulation(&vertex_indices);
            for triangle in triangles {
                if self.current_group.is_empty() {
                    self.default_group.add_child(
                        Object::Triangle(triangle),
                        crate::core::matrices::Matrix4::identity(),
                    );
                } else {
                    let group = self
                        .named_groups
                        .entry(self.current_group.clone())
                        .or_default();
                    group.add_child(
                        Object::Triangle(triangle),
                        crate::core::matrices::Matrix4::identity(),
                    );
                }
            }
        }
    }

    /// Parse a group line (g group_name)
    fn parse_group(&mut self, parts: &[&str]) {
        if parts.is_empty() {
            self.current_group = String::new();
        } else {
            self.current_group = parts.join(" ");
            // Ensure the group exists
            self.named_groups
                .entry(self.current_group.clone())
                .or_default();
        }
    }

    /// Fan triangulation: convert a polygon to triangles
    /// Uses the first vertex as the pivot point
    fn fan_triangulation(&self, indices: &[usize]) -> Vec<Triangle> {
        let mut triangles = Vec::new();

        if indices.len() < 3 {
            return triangles;
        }

        // Get the first vertex (pivot)
        let p1 = self.vertices[indices[0]];

        // Create triangles by fanning from the first vertex
        for i in 1..(indices.len() - 1) {
            let p2 = self.vertices[indices[i]];
            let p3 = self.vertices[indices[i + 1]];
            triangles.push(Triangle::new(p1, p2, p3));
        }

        triangles
    }

    /// Fan triangulation for smooth triangles with normals
    /// Uses the first vertex as the pivot point
    fn fan_triangulation_smooth(
        &self,
        vertex_indices: &[usize],
        normal_indices: &[Option<usize>],
    ) -> Vec<SmoothTriangle> {
        let mut triangles = Vec::new();

        if vertex_indices.len() < 3 {
            return triangles;
        }

        // Get the first vertex and normal (pivot)
        let p1 = self.vertices[vertex_indices[0]];
        let n1 = self.normals[normal_indices[0].unwrap()];

        // Create smooth triangles by fanning from the first vertex
        for i in 1..(vertex_indices.len() - 1) {
            let p2 = self.vertices[vertex_indices[i]];
            let p3 = self.vertices[vertex_indices[i + 1]];
            let n2 = self.normals[normal_indices[i].unwrap()];
            let n3 = self.normals[normal_indices[i + 1].unwrap()];
            triangles.push(SmoothTriangle::new(p1, p2, p3, n1, n2, n3));
        }

        triangles
    }
}

impl Default for ObjParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse an OBJ file from a string
/// Uses a two-pass approach: first pass loads vertices and normals,
/// second pass creates faces (which may reference normals loaded in first pass)
pub fn parse_obj_file(content: &str) -> ObjParser {
    let mut parser = ObjParser::new();

    // First pass: load vertices and normals
    for line in content.lines() {
        parser.parse_line_first_pass(line);
    }

    // Reset current_group before second pass to avoid mis-grouping
    parser.current_group = String::new();

    // Second pass: create faces
    for line in content.lines() {
        parser.parse_line_second_pass(line);
    }

    parser
}

/// Convert a parsed OBJ file to a single Group containing all groups
pub fn obj_to_group(parser: &ObjParser) -> Group {
    let mut group = Group::new();

    // Add default group if it has children
    if !parser.default_group.is_empty() {
        group.add_child(
            Object::Group(parser.default_group.clone()),
            crate::core::matrices::Matrix4::identity(),
        );
    }

    // Add all named groups
    for named_group in parser.named_groups.values() {
        if !named_group.is_empty() {
            // Clone the group and add it
            group.add_child(
                Object::Group(named_group.clone()),
                crate::core::matrices::Matrix4::identity(),
            );
        }
    }

    group
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tuples::Tuple;

    #[test]
    fn ignoring_unrecognized_lines() {
        let gibberish = r#"
There was a young lady named Bright
who traveled much faster than light.
She set out one day
in a relative way,
and came back the previous night.
"#;
        let parser = parse_obj_file(gibberish);
        assert_eq!(parser.ignored_lines, 5);
    }

    #[test]
    fn vertex_records() {
        let file = r#"
v -1 1 0
v -1.0000 0.5000 0.0000
v 1 0 0
v 1 1 0
"#;
        let parser = parse_obj_file(file);
        assert_eq!(parser.vertices[1], Point::new(-1.0, 1.0, 0.0));
        assert_eq!(parser.vertices[2], Point::new(-1.0, 0.5, 0.0));
        assert_eq!(parser.vertices[3], Point::new(1.0, 0.0, 0.0));
        assert_eq!(parser.vertices[4], Point::new(1.0, 1.0, 0.0));
    }

    #[test]
    fn parsing_triangle_faces() {
        let file = r#"
v -1 1 0
v -1 0 0
v 1 0 0
v 1 1 0

f 1 2 3
f 1 3 4
"#;
        let parser = parse_obj_file(file);
        let g = &parser.default_group;
        let t1 = &g.children()[0];
        let t2 = &g.children()[1];

        if let Object::Triangle(tri1) = t1 {
            assert_eq!(tri1.p1, parser.vertices[1]);
            assert_eq!(tri1.p2, parser.vertices[2]);
            assert_eq!(tri1.p3, parser.vertices[3]);
        } else {
            panic!("Expected triangle");
        }

        if let Object::Triangle(tri2) = t2 {
            assert_eq!(tri2.p1, parser.vertices[1]);
            assert_eq!(tri2.p2, parser.vertices[3]);
            assert_eq!(tri2.p3, parser.vertices[4]);
        } else {
            panic!("Expected triangle");
        }
    }

    #[test]
    fn triangulating_polygons() {
        let file = r#"
v -1 1 0
v -1 0 0
v 1 0 0
v 1 1 0
v 0 2 0

f 1 2 3 4 5
"#;
        let parser = parse_obj_file(file);
        let g = &parser.default_group;
        let t1 = &g.children()[0];
        let t2 = &g.children()[1];
        let t3 = &g.children()[2];

        if let Object::Triangle(tri1) = t1 {
            assert_eq!(tri1.p1, parser.vertices[1]);
            assert_eq!(tri1.p2, parser.vertices[2]);
            assert_eq!(tri1.p3, parser.vertices[3]);
        } else {
            panic!("Expected triangle");
        }

        if let Object::Triangle(tri2) = t2 {
            assert_eq!(tri2.p1, parser.vertices[1]);
            assert_eq!(tri2.p2, parser.vertices[3]);
            assert_eq!(tri2.p3, parser.vertices[4]);
        } else {
            panic!("Expected triangle");
        }

        if let Object::Triangle(tri3) = t3 {
            assert_eq!(tri3.p1, parser.vertices[1]);
            assert_eq!(tri3.p2, parser.vertices[4]);
            assert_eq!(tri3.p3, parser.vertices[5]);
        } else {
            panic!("Expected triangle");
        }
    }

    #[test]
    fn triangles_in_groups() {
        let file = r#"
v -1 1 0
v -1 0 0
v 1 0 0
v 1 1 0

g FirstGroup
f 1 2 3
g SecondGroup
f 1 3 4
"#;
        let parser = parse_obj_file(file);
        let g1 = parser.get_group("FirstGroup").unwrap();
        let g2 = parser.get_group("SecondGroup").unwrap();
        let t1 = &g1.children()[0];
        let t2 = &g2.children()[0];

        if let Object::Triangle(tri1) = t1 {
            assert_eq!(tri1.p1, parser.vertices[1]);
            assert_eq!(tri1.p2, parser.vertices[2]);
            assert_eq!(tri1.p3, parser.vertices[3]);
        } else {
            panic!("Expected triangle");
        }

        if let Object::Triangle(tri2) = t2 {
            assert_eq!(tri2.p1, parser.vertices[1]);
            assert_eq!(tri2.p2, parser.vertices[3]);
            assert_eq!(tri2.p3, parser.vertices[4]);
        } else {
            panic!("Expected triangle");
        }
    }

    #[test]
    fn converting_obj_file_to_group() {
        let file = r#"
v -1 1 0
v -1 0 0
v 1 0 0
v 1 1 0

g FirstGroup
f 1 2 3
g SecondGroup
f 1 3 4
"#;
        let parser = parse_obj_file(file);
        let g = obj_to_group(&parser);

        // The group should contain two named groups as children
        assert_eq!(g.children().len(), 2);

        // Verify the children are groups
        if let Object::Group(_) = &g.children()[0] {
            // Expected
        } else {
            panic!("Expected first child to be a group");
        }

        if let Object::Group(_) = &g.children()[1] {
            // Expected
        } else {
            panic!("Expected second child to be a group");
        }
    }

    #[test]
    fn vertex_normal_records() {
        let file = r#"
vn 0 0 1
vn 0.707 0 -0.707
vn 1 2 3
"#;
        let parser = parse_obj_file(file);
        assert_eq!(parser.normals[1], Vector::new(0.0, 0.0, 1.0));
        assert_eq!(parser.normals[2], Vector::new(0.707, 0.0, -0.707));
        assert_eq!(parser.normals[3], Vector::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn faces_with_normals() {
        let file = r#"
v 0 1 0
v -1 0 0
v 1 0 0

vn -1 0 0
vn 1 0 0
vn 0 1 0

f 1//3 2//1 3//2
f 1/0/3 2/102/1 3/14/2
"#;
        let parser = parse_obj_file(file);
        let g = &parser.default_group;
        let t1 = &g.children()[0];
        let t2 = &g.children()[1];

        if let Object::SmoothTriangle(tri1) = t1 {
            assert_eq!(tri1.p1, parser.vertices[1]);
            assert_eq!(tri1.p2, parser.vertices[2]);
            assert_eq!(tri1.p3, parser.vertices[3]);
            assert_eq!(tri1.n1, parser.normals[3]);
            assert_eq!(tri1.n2, parser.normals[1]);
            assert_eq!(tri1.n3, parser.normals[2]);
        } else {
            panic!("Expected smooth triangle");
        }

        if let Object::SmoothTriangle(tri2) = t2 {
            assert_eq!(tri2.p1, parser.vertices[1]);
            assert_eq!(tri2.p2, parser.vertices[2]);
            assert_eq!(tri2.p3, parser.vertices[3]);
            assert_eq!(tri2.n1, parser.normals[3]);
            assert_eq!(tri2.n2, parser.normals[1]);
            assert_eq!(tri2.n3, parser.normals[2]);
        } else {
            panic!("Expected smooth triangle");
        }
    }

    #[test]
    fn normals_at_end_of_file() {
        // Test the two-pass parsing: normals at end, faces at beginning
        let file = r#"
v 0 1 0
v -1 0 0
v 1 0 0

f 1//3 2//1 3//2

vn -1 0 0
vn 1 0 0
vn 0 1 0
"#;
        let parser = parse_obj_file(file);
        let g = &parser.default_group;
        let t1 = &g.children()[0];

        // Should still create a smooth triangle even though normals come after faces
        if let Object::SmoothTriangle(tri1) = t1 {
            assert_eq!(tri1.p1, parser.vertices[1]);
            assert_eq!(tri1.p2, parser.vertices[2]);
            assert_eq!(tri1.p3, parser.vertices[3]);
            assert_eq!(tri1.n1, parser.normals[3]);
            assert_eq!(tri1.n2, parser.normals[1]);
            assert_eq!(tri1.n3, parser.normals[2]);
        } else {
            panic!("Expected smooth triangle, got regular triangle - normals not loaded in time!");
        }
    }
}

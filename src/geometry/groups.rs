use crate::core::matrices::Matrix4;
use crate::core::tuples::{Point, Vector};
use crate::geometry::bounds::Bounds;
use crate::geometry::shapes::Shape;
use crate::rendering::objects::{HasMaterial, Intersectable, Object, Transformable};
use crate::rendering::rays::Ray;
use crate::scene::materials::Material;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// A Group is a collection of shapes that can be transformed together.
/// Groups support hierarchical transformations through parent-child relationships.
/// Children are stored directly to avoid lifetime issues with intersections.
#[derive(Debug, Clone)]
pub struct Group {
    pub transformation: Matrix4,
    pub world_transformation: Matrix4,
    pub material: Material,
    pub parent: Option<Weak<RefCell<Object>>>,
    children: Vec<Object>,
}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.transformation == other.transformation
            && self.material == other.material
            && self.children == other.children
        // Ignore parent for equality comparison
    }
}

impl Group {
    pub fn new() -> Self {
        Group {
            transformation: Matrix4::identity(),
            world_transformation: Matrix4::identity(),
            material: Material::default(),
            parent: None,
            children: Vec::new(),
        }
    }

    /// Add a child to the group, computing its world transform from the parent chain.
    /// The parent_world_transform is the accumulated world transform of this group's parent.
    pub fn add_child(&mut self, mut child: Object, parent_world_transform: Matrix4) {
        let child_world_transform =
            parent_world_transform * self.transformation * child.transformation();
        child.set_world_transform(child_world_transform);

        if let Object::Group(ref mut child_group) = child {
            let children: Vec<Object> = child_group.children.drain(..).collect();
            for grandchild in children {
                child_group.add_child(grandchild, parent_world_transform * self.transformation);
            }
        }

        self.children.push(child);
    }

    /// Check if this group is empty (has no children).
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Get a read-only view of the children.
    pub fn children(&self) -> &[Object] {
        &self.children
    }

    /// Update a child's transformation, maintaining correct world transforms.
    pub fn set_child_transform(&mut self, index: usize, transformation: Matrix4) {
        if let Some(child) = self.children.get_mut(index) {
            let child_world_transform = self.world_transformation * transformation;
            child.update_transforms(transformation, child_world_transform);
        }
    }

    /// Update a child's material.
    pub fn set_child_material(&mut self, index: usize, material: Material) {
        if let Some(child) = self.children.get_mut(index) {
            child.set_material(material);
        }
    }

    pub fn rebuild_children_transforms(&mut self, parent_world_transform: Matrix4) {
        for child in &mut self.children {
            let local_transform = child.transformation();
            let child_world_transform =
                parent_world_transform * self.transformation * local_transform;
            child.update_transforms(local_transform, child_world_transform);
        }
    }
}

/// Helper to recursively update world transforms for a group's children.
pub(crate) fn propagate_world_transform_to_group_children(
    group: &mut Group,
    parent_world_transform: Matrix4,
) {
    for child in &mut group.children {
        let local_transform = child.transformation();
        let child_world_transform = parent_world_transform * local_transform;
        child.update_transforms(local_transform, child_world_transform);
    }
}

impl Shape for Group {
    fn local_intersect(&self, ray: Ray) -> Vec<f64> {
        let group_bounds = self.bounds();
        if !group_bounds.intersects(ray) {
            return vec![];
        }

        let mut all_intersections = Vec::new();
        for child in &self.children {
            let child_intersections = child.intersect(ray);
            all_intersections.extend(child_intersections);
        }

        all_intersections
            .sort_by(|a: &f64, b: &f64| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        all_intersections
    }

    fn local_normal_at(&self, _point: Point) -> Vector {
        panic!("Groups do not have a normal - normal_at should be called on child objects")
    }

    /// Get the bounding box for this group by transforming and combining child bounds.
    fn bounds(&self) -> Bounds {
        let mut group_bounds = Bounds::empty();

        for child in &self.children {
            let child_bounds = match child {
                Object::Sphere(s) => s.bounds(),
                Object::Plane(p) => p.bounds(),
                Object::Cube(c) => c.bounds(),
                Object::Cylinder(cy) => cy.bounds(),
                Object::Cone(co) => co.bounds(),
                Object::Group(g) => g.bounds(),
            };

            let child_transform = child.transformation();
            let transformed_bounds = child_bounds.transform(child_transform);
            group_bounds = group_bounds.merge(&transformed_bounds);
        }

        group_bounds
    }
}

impl Default for Group {
    fn default() -> Self {
        Self::new()
    }
}

/// Add a child object to a group (convenience wrapper).
pub fn add_child_to_group(group: &mut Group, child: Object) {
    group.add_child(child, Matrix4::identity());
}

/// Legacy function for Rc<RefCell<>> based groups (used by tests with parent pointers).
pub fn add_child_to_group_rc(group: &Rc<RefCell<Object>>, child: Rc<RefCell<Object>>) {
    child.borrow_mut().set_parent(Rc::downgrade(group));

    let parent_world_transform = {
        let group_obj = group.borrow();
        if let Some(parent_weak) = group_obj.parent() {
            if let Some(parent_rc) = parent_weak.upgrade() {
                parent_rc.borrow().world_transformation()
            } else {
                Matrix4::identity()
            }
        } else {
            Matrix4::identity()
        }
    };

    if let Object::Group(ref mut g) = *group.borrow_mut() {
        let child_world_transform =
            parent_world_transform * g.transformation * child.borrow().transformation();
        let child_obj = (*child.borrow()).clone();
        g.add_child(child_obj, parent_world_transform);

        // Update the original Rc reference so operations on it see the correct world transform
        child
            .borrow_mut()
            .set_world_transform(child_world_transform);
    } else {
        panic!("add_child_to_group_rc called on non-group object");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::matrices::Matrix4;
    use crate::rendering::objects::Object;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn creating_a_new_group() {
        let g = Group::new();
        assert_eq!(g.transformation, Matrix4::identity());
        assert!(g.is_empty());
    }

    #[test]
    fn shape_has_parent_attribute() {
        let s = Object::sphere();
        assert!(s.parent().is_none());
    }

    #[test]
    fn adding_child_to_group() {
        let g = Rc::new(RefCell::new(Object::group()));
        let s = Rc::new(RefCell::new(Object::sphere()));

        add_child_to_group_rc(&g, s.clone());

        if let Object::Group(ref group) = *g.borrow() {
            assert!(!group.is_empty());
        } else {
            panic!("Expected group");
        }

        assert!(s.borrow().parent().is_some());
    }

    #[test]
    fn intersecting_ray_with_empty_group() {
        use crate::core::tuples::Tuple;
        use crate::rendering::objects::Intersectable;

        let g = Object::group();
        let r = Ray::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0));
        let xs = g.intersect(r);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn intersecting_ray_with_nonempty_group() {
        use crate::core::tuples::Tuple;
        use crate::rendering::objects::{Intersectable, Transformable};

        let g = Rc::new(RefCell::new(Object::group()));
        let s1 = Rc::new(RefCell::new(Object::sphere()));
        let s2 = Rc::new(RefCell::new(Object::sphere()));
        s2.borrow_mut()
            .set_transform(Matrix4::translate(0.0, 0.0, -3.0));
        let s3 = Rc::new(RefCell::new(Object::sphere()));
        s3.borrow_mut()
            .set_transform(Matrix4::translate(5.0, 0.0, 0.0));

        add_child_to_group_rc(&g, s1);
        add_child_to_group_rc(&g, s2);
        add_child_to_group_rc(&g, s3);

        let r = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));
        let xs = g.borrow().intersect(r);

        assert_eq!(xs.len(), 4);
        // Intersections should be sorted by t value
        // s2 is at z=-3, so it's closer: intersects at t=1 and t=3
        // s1 is at z=0: intersects at t=4 and t=6
        // s3 is at x=5, so the ray misses it
        assert_eq!(xs[0], 1.0); // s2
        assert_eq!(xs[1], 3.0); // s2
        assert_eq!(xs[2], 4.0); // s1
        assert_eq!(xs[3], 6.0); // s1
    }

    #[test]
    fn intersecting_transformed_group() {
        use crate::core::tuples::Tuple;
        use crate::rendering::objects::{Intersectable, Transformable};

        let g = Rc::new(RefCell::new(Object::group()));
        g.borrow_mut().set_transform(Matrix4::scale(2.0, 2.0, 2.0));

        let s = Rc::new(RefCell::new(Object::sphere()));
        s.borrow_mut()
            .set_transform(Matrix4::translate(5.0, 0.0, 0.0));

        add_child_to_group_rc(&g, s);

        let r = Ray::new(Point::new(10.0, 0.0, -10.0), Vector::new(0.0, 0.0, 1.0));
        let xs = g.borrow().intersect(r);

        assert_eq!(xs.len(), 2);
    }

    #[test]
    fn converting_point_from_world_to_object_space() {
        use crate::core::tuples::Tuple;
        use crate::rendering::objects::Transformable;
        use std::f64::consts::PI;

        let g1 = Rc::new(RefCell::new(Object::group()));
        g1.borrow_mut().set_transform(Matrix4::rotate_y(PI / 2.0));

        let g2 = Rc::new(RefCell::new(Object::group()));
        g2.borrow_mut().set_transform(Matrix4::scale(2.0, 2.0, 2.0));

        add_child_to_group_rc(&g1, g2.clone());

        let s = Rc::new(RefCell::new(Object::sphere()));
        s.borrow_mut()
            .set_transform(Matrix4::translate(5.0, 0.0, 0.0));

        add_child_to_group_rc(&g2, s.clone());

        let p = s.borrow().world_to_object(Point::new(-2.0, 0.0, -10.0));
        assert_eq!(p, Point::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn converting_normal_from_object_to_world_space() {
        use crate::core::tuples::Tuple;
        use crate::rendering::objects::Transformable;
        use std::f64::consts::PI;

        let g1 = Rc::new(RefCell::new(Object::group()));
        g1.borrow_mut().set_transform(Matrix4::rotate_y(PI / 2.0));

        let g2 = Rc::new(RefCell::new(Object::group()));
        g2.borrow_mut().set_transform(Matrix4::scale(1.0, 2.0, 3.0));

        add_child_to_group_rc(&g1, g2.clone());

        let s = Rc::new(RefCell::new(Object::sphere()));
        s.borrow_mut()
            .set_transform(Matrix4::translate(5.0, 0.0, 0.0));

        add_child_to_group_rc(&g2, s.clone());

        let sqrt3_over_3 = 3.0_f64.sqrt() / 3.0;
        let n = s
            .borrow()
            .normal_to_world(Vector::new(sqrt3_over_3, sqrt3_over_3, sqrt3_over_3));

        // Expected: (0.2857, 0.4286, -0.8571)
        assert!((n.x - 0.2857).abs() < 0.0001);
        assert!((n.y - 0.4286).abs() < 0.0001);
        assert!((n.z - (-0.8571)).abs() < 0.0001);
    }

    #[test]
    fn finding_normal_on_child_object() {
        use crate::core::tuples::Tuple;
        use crate::rendering::objects::{Intersectable, Transformable};
        use std::f64::consts::PI;

        let g1 = Rc::new(RefCell::new(Object::group()));
        g1.borrow_mut().set_transform(Matrix4::rotate_y(PI / 2.0));

        let g2 = Rc::new(RefCell::new(Object::group()));
        g2.borrow_mut().set_transform(Matrix4::scale(1.0, 2.0, 3.0));

        add_child_to_group_rc(&g1, g2.clone());

        let s = Rc::new(RefCell::new(Object::sphere()));
        s.borrow_mut()
            .set_transform(Matrix4::translate(5.0, 0.0, 0.0));

        add_child_to_group_rc(&g2, s.clone());

        let n = s.borrow().normal_at(Point::new(1.7321, 1.1547, -5.5774));

        // Expected: (0.2857, 0.4286, -0.8571)
        assert!((n.x - 0.2857).abs() < 0.0001);
        assert!((n.y - 0.4286).abs() < 0.0001);
        assert!((n.z - (-0.8571)).abs() < 0.0001);
    }

    #[test]
    fn bounds_of_empty_group() {
        use crate::geometry::shapes::Shape;

        let g = Group::new();
        let bounds = g.bounds();

        // Empty group should have empty bounds (inverted)
        // Check that min is infinity and max is negative infinity
        assert!(bounds.min.x.is_infinite() && bounds.min.x.is_sign_positive());
        assert!(bounds.min.y.is_infinite() && bounds.min.y.is_sign_positive());
        assert!(bounds.min.z.is_infinite() && bounds.min.z.is_sign_positive());
        assert!(bounds.max.x.is_infinite() && bounds.max.x.is_sign_negative());
        assert!(bounds.max.y.is_infinite() && bounds.max.y.is_sign_negative());
        assert!(bounds.max.z.is_infinite() && bounds.max.z.is_sign_negative());
    }

    #[test]
    fn bounds_of_group_with_children() {
        use crate::core::tuples::Tuple;
        use crate::geometry::shapes::Shape;
        use crate::rendering::objects::Transformable;

        let g = Rc::new(RefCell::new(Object::group()));

        // Add a sphere at the origin
        let s1 = Rc::new(RefCell::new(Object::sphere()));
        add_child_to_group_rc(&g, s1);

        // Add a sphere translated to (2, 0, 0)
        let s2 = Rc::new(RefCell::new(Object::sphere()));
        s2.borrow_mut()
            .set_transform(Matrix4::translate(2.0, 0.0, 0.0));
        add_child_to_group_rc(&g, s2);

        let bounds = if let Object::Group(ref group) = *g.borrow() {
            group.bounds()
        } else {
            panic!("Expected group");
        };

        // First sphere goes from -1 to 1 in all dimensions
        // Second sphere goes from 1 to 3 in x, -1 to 1 in y and z
        // Combined: x: -1 to 3, y: -1 to 1, z: -1 to 1
        assert_eq!(bounds.min, Point::new(-1.0, -1.0, -1.0));
        assert_eq!(bounds.max, Point::new(3.0, 1.0, 1.0));
    }

    #[test]
    fn bounds_of_transformed_group() {
        use crate::core::tuples::Tuple;
        use crate::geometry::shapes::Shape;
        use crate::rendering::objects::Transformable;

        let g = Rc::new(RefCell::new(Object::group()));
        g.borrow_mut().set_transform(Matrix4::scale(2.0, 2.0, 2.0));

        let s = Rc::new(RefCell::new(Object::sphere()));
        add_child_to_group_rc(&g, s);

        let bounds = if let Object::Group(ref group) = *g.borrow() {
            group.bounds()
        } else {
            panic!("Expected group");
        };

        // Sphere bounds in object space are -1 to 1
        // The group's bounds() returns bounds in the group's object space
        // which is still -1 to 1 (child's bounds, not transformed by group's transform)
        assert_eq!(bounds.min, Point::new(-1.0, -1.0, -1.0));
        assert_eq!(bounds.max, Point::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn bounds_optimization_skips_children_when_ray_misses() {
        use crate::core::tuples::Tuple;
        use crate::rendering::objects::Transformable;

        // Create a group with a sphere at (0, 0, -5)
        let g = Rc::new(RefCell::new(Object::group()));
        let s = Rc::new(RefCell::new(Object::sphere()));
        s.borrow_mut()
            .set_transform(Matrix4::translate(0.0, 0.0, -5.0));
        add_child_to_group_rc(&g, s);

        // Ray that completely misses the group's bounding box
        let ray = Ray::new(Point::new(10.0, 10.0, 10.0), Vector::new(1.0, 0.0, 0.0));

        let xs = g.borrow().intersect(ray);

        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn bounds_optimization_tests_children_when_ray_hits() {
        use crate::core::tuples::Tuple;
        use crate::rendering::objects::Intersectable;

        let g = Rc::new(RefCell::new(Object::group()));
        let s = Rc::new(RefCell::new(Object::sphere()));
        add_child_to_group_rc(&g, s);

        // Ray that hits the bounding box and the sphere
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));

        let xs = g.borrow().intersect(ray);

        assert_eq!(xs.len(), 2);
    }

    #[test]
    fn bounds_of_nested_groups() {
        use crate::core::tuples::Tuple;
        use crate::geometry::shapes::Shape;
        use crate::rendering::objects::Transformable;

        // Create outer group
        let g1 = Rc::new(RefCell::new(Object::group()));

        // Create inner group translated to (2, 0, 0)
        let g2 = Rc::new(RefCell::new(Object::group()));
        g2.borrow_mut()
            .set_transform(Matrix4::translate(2.0, 0.0, 0.0));

        // Add a sphere to inner group
        let s = Rc::new(RefCell::new(Object::sphere()));
        add_child_to_group_rc(&g2, s);

        // Add inner group to outer group
        add_child_to_group_rc(&g1, g2);

        let bounds = if let Object::Group(ref group) = *g1.borrow() {
            group.bounds()
        } else {
            panic!("Expected group");
        };

        // Inner group has a sphere from -1 to 1 in its space
        // Inner group is translated by (2, 0, 0)
        // So in outer group space, sphere goes from (1, -1, -1) to (3, 1, 1)
        assert_eq!(bounds.min, Point::new(1.0, -1.0, -1.0));
        assert_eq!(bounds.max, Point::new(3.0, 1.0, 1.0));
    }

    #[test]
    fn intersecting_group_should_reference_child_objects_not_group() {
        use crate::core::tuples::Tuple;
        use crate::rendering::objects::Intersectable;

        // When we call intersect_with_object on a group, it should return
        // Intersection objects that reference the CHILD objects, not the group.
        // This allows us to shade the intersections by calling normal_at on
        // the actual child that was hit.

        let mut g = Object::group();
        let s = Object::sphere();

        if let Object::Group(ref mut group) = g {
            group.add_child(s, Matrix4::identity());
        }

        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector::new(0.0, 0.0, 1.0));

        let xs = g.intersect_with_object(ray);

        assert_eq!(xs.len(), 2);

        // xs[0].object should point to the CHILD (sphere), not the group
        // So calling normal_at should work and not panic
        if !xs.is_empty() {
            let hit_point = ray.position(xs[0].t);
            let normal = xs[0].object.normal_at(hit_point);

            assert_eq!(normal, Vector::new(0.0, 0.0, -1.0));
        }
    }

    #[test]
    fn set_transform_on_group_should_propagate_to_children() {
        use crate::core::tuples::Tuple;
        use crate::rendering::objects::Transformable;

        // When we call set_transform on a group AFTER children are already added,
        // the children's cached world_transformation should be updated.

        let mut g = Object::group();

        // Add a sphere with a translation (using non-Rc approach for simpler testing)
        let mut s = Object::sphere();
        s.set_transform(Matrix4::translate(5.0, 0.0, 0.0));

        if let Object::Group(ref mut group) = g {
            group.add_child(s, Matrix4::identity());
        }

        // At this point, sphere's world_transform is (translate 5,0,0)
        if let Object::Group(ref group) = g {
            let child = &group.children[0];
            let p1 = child.world_to_object(Point::new(5.0, 0.0, 0.0));
            assert_eq!(p1, Point::new(0.0, 0.0, 0.0));
        }

        // NOW change the group's transform - this should propagate to children
        g.set_transform(Matrix4::scale(2.0, 2.0, 2.0));

        // The sphere's world_transformation should now be scale(2,2,2) * translate(5,0,0)
        // which moves the sphere to (10, 0, 0) in world space
        if let Object::Group(ref group) = g {
            let child = &group.children[0];
            let p2 = child.world_to_object(Point::new(10.0, 0.0, 0.0));
            assert_eq!(
                p2,
                Point::new(0.0, 0.0, 0.0),
                "World transform should be propagated to children"
            );
        }
    }

    #[test]
    fn updating_child_transform_should_preserve_parent_contribution() {
        use crate::core::tuples::Tuple;
        use crate::rendering::objects::Transformable;

        // When we update a child's transform after it's been added to a group,
        // the child's world_transformation should be parent_world * new_local_transform,
        // not just new_local_transform.

        let g = Rc::new(RefCell::new(Object::group()));
        g.borrow_mut()
            .set_transform(Matrix4::translate(10.0, 0.0, 0.0)); // Group at x=10

        let s = Rc::new(RefCell::new(Object::sphere()));
        s.borrow_mut()
            .set_transform(Matrix4::translate(1.0, 0.0, 0.0)); // Sphere at x=1 relative to group
        add_child_to_group_rc(&g, s.clone());

        // Sphere should be at x=11 in world space (10 + 1)
        let p1 = s.borrow().world_to_object(Point::new(11.0, 0.0, 0.0));
        assert_eq!(
            p1,
            Point::new(0.0, 0.0, 0.0),
            "Initial world transform correct"
        );

        // Now update the sphere's transform to translate(2, 0, 0)
        s.borrow_mut()
            .set_transform(Matrix4::translate(2.0, 0.0, 0.0));

        // The sphere should now be at x=12 in world space (10 + 2)
        let p2 = s.borrow().world_to_object(Point::new(12.0, 0.0, 0.0));
        assert_eq!(
            p2,
            Point::new(0.0, 0.0, 0.0),
            "World transform should include parent contribution after update"
        );
    }

    #[test]
    fn updating_nested_group_transform_should_preserve_parent_contribution() {
        use crate::core::tuples::Tuple;
        use crate::rendering::objects::Transformable;

        // When we update a nested group's transform, it should preserve its parent's
        // world transform and propagate the correct combined transform to descendants.

        let g1 = Rc::new(RefCell::new(Object::group()));
        g1.borrow_mut()
            .set_transform(Matrix4::translate(10.0, 0.0, 0.0)); // Parent group at x=10

        let g2 = Rc::new(RefCell::new(Object::group()));
        g2.borrow_mut()
            .set_transform(Matrix4::translate(1.0, 0.0, 0.0)); // Child group at x=1 relative to parent
        add_child_to_group_rc(&g1, g2.clone());

        let s = Rc::new(RefCell::new(Object::sphere()));
        s.borrow_mut()
            .set_transform(Matrix4::translate(0.5, 0.0, 0.0)); // Sphere at x=0.5 relative to child group
        add_child_to_group_rc(&g2, s.clone());

        // Sphere should be at x=11.5 in world space (10 + 1 + 0.5)
        {
            let g2_borrow = g2.borrow();
            let child_sphere = if let Object::Group(ref g2_inner) = *g2_borrow {
                &g2_inner.children[0]
            } else {
                panic!("Expected group");
            };
            let p1 = child_sphere.world_to_object(Point::new(11.5, 0.0, 0.0));
            assert_eq!(
                p1,
                Point::new(0.0, 0.0, 0.0),
                "Initial nested world transform correct"
            );
        }

        // Now update g2's transform to translate(2, 0, 0)
        g2.borrow_mut()
            .set_transform(Matrix4::translate(2.0, 0.0, 0.0));

        // The sphere should now be at x=12.5 in world space (10 + 2 + 0.5)
        {
            let g2_borrow = g2.borrow();
            let child_sphere = if let Object::Group(ref g2_inner) = *g2_borrow {
                &g2_inner.children[0]
            } else {
                panic!("Expected group");
            };
            let p2 = child_sphere.world_to_object(Point::new(12.5, 0.0, 0.0));
            assert_eq!(
                p2,
                Point::new(0.0, 0.0, 0.0),
                "Nested group update should preserve parent contribution"
            );
        }
    }

    #[test]
    fn set_child_transform_maintains_world_transform() {
        use crate::core::tuples::Tuple;

        let mut g = Object::group();

        if let Object::Group(ref mut group) = g {
            group.transformation = Matrix4::translate(5.0, 0.0, 0.0);
            group.world_transformation = Matrix4::translate(5.0, 0.0, 0.0);
        }

        let s = Object::sphere();
        if let Object::Group(ref mut group) = g {
            group.add_child(s, Matrix4::identity());
        }

        if let Object::Group(ref mut group) = g {
            group.set_child_transform(0, Matrix4::translate(2.0, 0.0, 0.0));
        }

        // Sphere should be at x=7 in world space (5 + 2)
        if let Object::Group(ref group) = g {
            let child = &group.children[0];
            let p = child.world_to_object(Point::new(7.0, 0.0, 0.0));
            assert_eq!(
                p,
                Point::new(0.0, 0.0, 0.0),
                "set_child_transform should maintain correct world transform"
            );
        }
    }

    #[test]
    fn set_child_material_updates_child() {
        use crate::core::color::Color;
        use crate::rendering::objects::HasMaterial;

        let mut g = Object::group();
        let s = Object::sphere();

        if let Object::Group(ref mut group) = g {
            group.add_child(s, Matrix4::identity());
        }

        let new_material = Material::builder()
            .color(Color::new(1.0, 0.0, 0.0))
            .diffuse(0.5)
            .build();

        if let Object::Group(ref mut group) = g {
            group.set_child_material(0, new_material.clone());
        }

        if let Object::Group(ref group) = g {
            let child = &group.children[0];
            assert_eq!(child.material().color, Color::new(1.0, 0.0, 0.0));
            assert_eq!(child.material().diffuse, 0.5);
        }
    }

    #[test]
    fn set_child_transform_propagates_to_nested_groups() {
        use crate::core::tuples::Tuple;

        let mut g1 = Object::group();

        if let Object::Group(ref mut group) = g1 {
            group.world_transformation = Matrix4::translate(10.0, 0.0, 0.0);
        }

        let mut g2 = Object::group();
        let s = Object::sphere();
        if let Object::Group(ref mut group) = g2 {
            group.add_child(s, Matrix4::identity());
        }

        if let Object::Group(ref mut group) = g1 {
            group.add_child(g2, Matrix4::identity());
        }

        if let Object::Group(ref mut group) = g1 {
            group.set_child_transform(0, Matrix4::translate(2.0, 0.0, 0.0));
        }

        // g1 at x=10, g2 at x=2 relative to g1, sphere at origin relative to g2
        // So sphere should be at x=12 in world space
        if let Object::Group(ref group) = g1 {
            if let Object::Group(ref g2_inner) = group.children[0] {
                let sphere = &g2_inner.children[0];
                let p = sphere.world_to_object(Point::new(12.0, 0.0, 0.0));
                assert_eq!(
                    p,
                    Point::new(0.0, 0.0, 0.0),
                    "set_child_transform should propagate to nested groups"
                );
            }
        }
    }
}

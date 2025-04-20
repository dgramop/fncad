use std::collections::BTreeMap;
type Point3 = nalgebra::Point3<f32>;
//TODO: boolean, epxlicit parameters

// A fully specified shape we pass to manifoldcad, ready for rendering
// TODO(Dhruv) include IDs here so troy can send them back when users force points in the ui
pub enum DeterminedShape {
    Point(Point3),
    Line(Point3, Point3),
    Circle {
        radius: f32,
        origin: Point3
    }
}

type Id = usize;

pub type RenderableScene = BTreeMap<Id, DeterminedShape>;

// Give troy: map Id -> DeterminedShape

// Give everyone IDs (TODO)

// A relation that is passed to the solver
pub struct Relation;

// An expression that a user types in
//struct Expr;
//

// stopgap until we add user-defined expression support
pub type Expr = f32;
pub type Parameter = f32;


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PointId(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CircleId(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SegmentId(pub usize);

pub struct Point {
    pub x: Parameter,
    pub y: Parameter,
    pub z: Parameter,
}

pub struct Segment {
    pub a: PointId,
    pub b: PointId
}

pub struct Circle {
    pub radius: Parameter,
    pub origin: PointId
}

pub enum Constraint {
    PointOnCircle(PointId, CircleId),
    PointOnLine(PointId, SegmentId),
    Distance(PointId, PointId, Expr),
}

#[derive(Default)]
pub struct Objects {
    pub points: BTreeMap<PointId, Point>,
    pub segments: BTreeMap<SegmentId, Segment>,
    pub circles: BTreeMap<CircleId, Circle>,
}



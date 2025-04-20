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
pub struct Parameter {
    /// Current value of this parameter
    pub value: f32,

    /// If the initial value is locked (for example, because a user has overriden it, or because
    /// this CAD model is being called with this parameter given as an input)
    pub locked: bool
}

impl From<f32> for Parameter {
    fn from(value: f32) -> Self {
        Self {
            value,
            locked: false
        }
    }
}

impl Parameter {
    pub fn fixed(value: f32) -> Self {
        Self {
            value,
            locked: true
        }
    }

    pub fn free(value: f32) -> Self {
        Self {
            value,
            locked: false
        }
    }
}

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
    pub origin: PointId,
    pub radius: Parameter
}

pub enum Constraint {
    PointOnCircle(PointId, CircleId),
    PointOnLine(PointId, SegmentId),
    Distance(PointId, PointId, Expr),
}

#[derive(Default)]
pub struct Objects {
    // if everyone has their own Points and we don't actually reuse it,
    // is there any point in having this live in a map/vec?
    // maybe only parameters live in the map at that point
    // This could reduce indirection and save us from cargo culting solvespace
    pub points: Vec<Point>,
    pub segments: Vec<Segment>,
    pub circles: Vec<Circle>,
}


//TODO: for a point on point constraing, we can delete the original point and substitute its use
//everywhere (the problem is if the file is included and this point is "part of the public API".
//Instead, we should maybe perform this before solving for it to reduce the number of parameters

impl Objects {
    pub fn get_point(&self, id: PointId) -> Option<&Point> {
        self.points.get(id.0)
    }
    
    pub fn get_circle(&mut self, id: CircleId) -> Option<&Circle> {
        self.circles.get(id.0)
    }

    pub fn add_point(&mut self, x: Parameter, y: Parameter, z: Parameter) -> PointId {
        self.points.push(Point {
            x,
            y,
            z
        });

        return PointId(self.points.len()-1);
    }

    // PtOnPt deduplication of parameters. Propogate deduplication up
    // for linking sketches, only some things will be available (like constraint to surface).
    // Otherwise, they would have to manually export specific things from linking 

    pub fn add_circle(&mut self, origin: PointId, radius: Parameter) -> CircleId {
        self.circles.push(Circle {
            origin,
            radius 
        });
        return CircleId(self.circles.len()-1);
    }
}

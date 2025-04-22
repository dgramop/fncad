use std::collections::BTreeMap;

use argmin::{core::{CostFunction, Executor, Jacobian, Operator}, solver::{gaussnewton::GaussNewton, newton::Newton}};
use cobyla::{minimize, RhoBeg};
use nalgebra::{constraint, DMatrix, DVector};
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
    },
    Sphere {
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

#[derive(Clone)]
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
pub struct ParameterId(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PointId(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CircleId(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SegmentId(pub usize);

#[derive(Clone)]
pub struct Point {
    pub x: ParameterId,
    pub y: ParameterId,
    pub z: ParameterId,
}

#[derive(Clone)]
pub struct Segment {
    pub a: PointId,
    pub b: PointId
}

#[derive(Clone)]
pub struct Circle {
    pub origin: PointId,
    pub radius: ParameterId
}

pub enum Constraint {
    PointOnCircle(PointId, CircleId),
    PointOnLine(PointId, SegmentId),
    Distance(PointId, PointId, Expr),
}

#[derive(Default, Clone)]
pub struct Objects {
    // if everyone has their own Points/Parameters and we don't actually reuse it,
    // is there any point in having this live in a map/vec?
    // maybe only parameters live in the map at that point
    // This could reduce indirection and save us from cargo culting solvespace
    pub parameters: Vec<Parameter>,
    pub points: Vec<Point>,
    pub segments: Vec<Segment>,
    pub circles: Vec<Circle>,
}

impl Objects {
    // iterator for parameters
}

//TODO: CAS to simplify this
struct Problem {
    // TODO: more intermediate stuff to make objects into expression-like things
    objects: Objects,
    constraints: Vec<Constraint>
}

//                          forced points -v
// constraints -> CAS and culling -> expressions <-> solver
impl Operator for Problem {
    // all the free var, corresponds to vec<parameter>
    type Param = DVector<f32>;

    type Output = DVector<f32>;

    fn apply(&self, param: &Self::Param) -> Result<Self::Output, argmin::core::Error> {
        // maybe a cleaner way to do lookups by looking up for an entire type at a time - like with
        // some kind of generics and with_dvector() conversion trait?
        let lookup = |id: ParameterId| {
            *param.get(id.0).expect("parameter id exists")
        };

        Ok(DVector::from_fn(self.constraints.len(), |constraint_index, _| {
            let constraint = &self.constraints[constraint_index];

            match constraint {
                Constraint::PointOnCircle(point, circle) => {
                    let point = self.objects.get_point(*point).unwrap();
                    let circle = self.objects.get_circle(*circle).unwrap();
                    let origin = self.objects.get_point(circle.origin).unwrap();

                    f32::abs(
                        f32::sqrt(
                            ((lookup)(point.x) - (lookup)(origin.x)).powi(2) + 
                            ((lookup)(point.y) - (lookup)(origin.y)).powi(2) +
                            ((lookup)(point.z) - (lookup)(origin.z)).powi(2)) 
                        - (lookup)(circle.radius)
                    )
                }
                _ => 0.0, //TODO(Dhruv) handle other constraint types
            }
        }))
    }
}

impl Jacobian for Problem {
    type Param = DVector<f32>;

    type Jacobian = DMatrix<f32>;

    //TODO: autodifferentiation
    fn jacobian(&self, param: &Self::Param) -> Result<Self::Jacobian, argmin::core::Error> {
        Ok(
            DMatrix::from_fn(self.constraints.len(), param.len(), |constraint_index, parameter_index| {
                let constraint = &self.constraints[constraint_index];
                match constraint {
                    Constraint::PointOnCircle(point_id, circle_id) => {
                        if point_id.x == parameter_index {
                        } else if point_id.y == parameter_index {
                        } else if point_id.z == parameter_index {
                        }

                    },
                    _ => 0.0 //TODO(Dhruv) handle 
                }
            })
        )
    }
}

//TODO: for a point on point constraing, we can delete the original point and substitute its use
//everywhere (the problem is if the file is included and this point is "part of the public API".
//Instead, we should maybe perform this before solving for it to reduce the number of parameters

impl Objects {
    /*pub fn get_parameter(&self, id: ParameterId) -> Option<&Parameter> {
        self.parameters.get(id.0)
    }*/

    pub fn get_point(&self, id: PointId) -> Option<&Point> {
        self.points.get(id.0)
    }
    
    pub fn get_circle(&self, id: CircleId) -> Option<&Circle> {
        self.circles.get(id.0)
    }

    pub fn add_parameter(&mut self, p: Parameter) -> ParameterId {
        self.parameters.push(p);
        return ParameterId(self.parameters.len() - 1)
    }

    pub fn add_point(&mut self, x: Parameter, y: Parameter, z: Parameter) -> PointId {
        self.points.push(Point {
            x: self.add_parameter(x),
            y: self.add_parameter(y),
            z: self.add_parameter(z)
        });

        return PointId(self.points.len()-1);
    }

    // PtOnPt deduplication of parameters. Propogate deduplication up
    // for linking sketches, only some things will be available (like constraint to surface).
    // Otherwise, they would have to manually export specific things from linking 

    pub fn add_circle(&mut self, origin: PointId, radius: Parameter) -> CircleId {
        self.circles.push(Circle {
            origin,
            radius: self.add_parameter(radius)
        });
        return CircleId(self.circles.len()-1);
    }
}

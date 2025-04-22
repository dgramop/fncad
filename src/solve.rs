use std::collections::BTreeMap;

use argmin::{core::{CostFunction, Executor, Gradient, Jacobian, Operator}, solver::{gaussnewton::GaussNewton, gradientdescent::SteepestDescent, linesearch::{BacktrackingLineSearch, MoreThuenteLineSearch}, newton::Newton}};
use cas_compute::{numerical::{ctxt::Ctxt, eval::Eval, value::Value}, symbolic::expr};
use cas_parser::parser::{ast::Expr, Parser};
use cobyla::{minimize, RhoBeg};
use nalgebra::{constraint, DMatrix, DVector};
type Point3 = nalgebra::Point3<f64>;
//TODO: boolean, epxlicit parameters

// A fully specified shape we pass to manifoldcad, ready for rendering
// TODO(Dhruv) include IDs here so troy can send them back when users force points in the ui
pub enum DeterminedShape {
    Point(Point3),
    Line(Point3, Point3),
    Circle {
        radius: f64,
        origin: Point3
    },
    Sphere {
        radius: f64,
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

#[derive(Clone)]
pub struct Parameter {
    /// Current value of this parameter
    pub value: f64,

    /// If the initial value is locked (for example, because a user has overriden it, or because
    /// this CAD model is being called with this parameter given as an input)
    pub locked: bool
}

impl From<f64> for Parameter {
    fn from(value: f64) -> Self {
        Self {
            value,
            locked: false
        }
    }
}

impl Parameter {
    pub fn fixed(value: f64) -> Self {
        Self {
            value,
            locked: true
        }
    }

    pub fn free(value: f64) -> Self {
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
pub struct Problem {
    // TODO: more intermediate stuff to make objects into expression-like things
    pub objects: Objects,
    pub constraints: Vec<Constraint>
}

impl Problem {
    pub fn solve(self) {
        // Set up solver
        //let solver: GaussNewton<f64> = GaussNewton::new();
        let solver = SteepestDescent::new(MoreThuenteLineSearch::new());

        let params = DVector::from_iterator(self.objects.parameters.len(), self.objects.parameters.iter().map(|p| p.value));

        // Run solver
        let res = Executor::new(self, solver)
            .configure(|state| state.param(params).max_iters(10))
            .run().unwrap();

        // Print result
        println!("{res}");
    }
}

struct CtxtBuilder<'a> {
    objects: &'a Objects,
    guess: &'a DVector<f64>,
    ctxt: Ctxt
}


impl<'a> CtxtBuilder<'a> {
    fn new(objects: &'a Objects, guess: &'a DVector<f64>) -> Self {
        Self {
            objects,
            guess,
            ctxt: Ctxt::default()
        }
    }
    
    //TODO: builder/just put in semi-anonymous stuff
    fn put(&mut self, name: &'static str, id: ParameterId) {
        if let Parameter { value, locked: true } = self.objects.get_parameter(id).expect("parameter exists") {
            // it's locked, don't use the solved version
            self.ctxt.add_var(name, (*value as f64).into());
        } else {
            // we're solving for this, use the guess
            self.ctxt.add_var(name, (self.guess[id.0] as f64).into());
        }
    }

}

impl<'a> From<CtxtBuilder<'a>> for Ctxt {
    fn from(value: CtxtBuilder) -> Self {
        value.ctxt
    }
}


//                          forced points -v
// constraints -> CAS and culling -> expressions <-> solver
impl Operator for Problem {
    // all the free var, corresponds to vec<parameter>
    type Param = DVector<f64>;

    type Output = DVector<f64>;

    fn apply(&self, param: &Self::Param) -> Result<Self::Output, argmin::core::Error> {
        // maybe a cleaner way to do lookups by looking up for an entire type at a time - like with
        // some kind of generics and with_dvector() conversion trait?

        println!("len: {:?}", param.len());

        let guess = Ok(DVector::from_fn(self.constraints.len(), |constraint_index, _| {
            let constraint = &self.constraints[constraint_index];

            match constraint {
                Constraint::PointOnCircle(point, circle) => {
                    let point = self.objects.get_point(*point).unwrap();
                    let circle = self.objects.get_circle(*circle).unwrap();
                    let origin = self.objects.get_point(circle.origin).unwrap();

                    let mut ctxt = CtxtBuilder::new(&self.objects, param);
                    ctxt.put("x", point.x);
                    ctxt.put("y", point.y);
                    ctxt.put("z", point.z);

                    ctxt.put("a", origin.x);
                    ctxt.put("b", origin.y);
                    ctxt.put("c", origin.z);

                    ctxt.put("r", circle.radius);

                    let cost = Parser::new("sqrt((x-a)^2 + (y-b)^2 + (z-c)^2) - r").try_parse_full::<Expr>().unwrap();
                    if let Value::Float(ans) = cost.eval(&mut ctxt.into()).expect("equation is good").coerce_float() {
                        ans.to_f64()
                    } else {
                        // we're cooked
                        0.0
                    }
                }
                _ => 0.0, //TODO(Dhruv) handle other constraint types
            }
        }));

        println!("guess is off by {guess:?}");

        guess
    }
}

impl Jacobian for Problem {
    type Param = DVector<f64>;

    type Jacobian = DMatrix<f64>;

    //TODO: autodifferentiation
    fn jacobian(&self, param: &Self::Param) -> Result<Self::Jacobian, argmin::core::Error> {
        let jacob = Ok(
            DMatrix::from_fn(self.constraints.len(), param.len(), |constraint_index, parameter_index| {
                let constraint = &self.constraints[constraint_index];

                //TODO: do not even include fixed parameters in the solver.
                if self.objects.get_parameter(ParameterId(parameter_index)).expect("TODO").locked {
                    return 0.;
                }

                match constraint {
                    Constraint::PointOnCircle(point_id, circle_id) => {
                        //TODO(Dhruv): generic into iterable parameters?
                        let point = self.objects.get_point(*point_id).unwrap();
                        let circle = self.objects.get_circle(*circle_id).unwrap();
                        let origin = self.objects.get_point(circle.origin).unwrap();

                        // does this parameter appear inside this function?
                        let candidate = ParameterId(parameter_index);

                        let expr = if candidate == point.x {
                            "x - a / sqrt((x-a)^2 + (y-b)^2 + (z-c)^2)"
                        } else if candidate == point.y {
                            "y - b / sqrt((x-a)^2 + (y-b)^2 + (z-c)^2)"
                        } else if candidate == point.z {
                            "z - c / sqrt((x-a)^2 + (y-b)^2 + (z-c)^2)"
                        } else if candidate == origin.x {
                            "x - a / sqrt((x-a)^2 + (y-b)^2 + (z-c)^2)"
                        } else if candidate == origin.y {
                            "y - b / sqrt((x-a)^2 + (y-b)^2 + (z-c)^2)"
                        } else if candidate == origin.z {
                            "z - c / sqrt((x-a)^2 + (y-b)^2 + (z-c)^2)"
                        } else if candidate == circle.radius {
                            return -1.;
                        } else {
                            return 0.;
                        };

                        let mut ctxt = CtxtBuilder::new(&self.objects, param);

                        ctxt.put("x", point.x);
                        ctxt.put("y", point.y);
                        ctxt.put("z", point.z);

                        ctxt.put("a", origin.x);
                        ctxt.put("b", origin.y);
                        ctxt.put("c", origin.z);

                        ctxt.put("r", circle.radius);

                        let cost = Parser::new(expr).try_parse_full::<Expr>().unwrap(); //d/dx and friends
                        if let Value::Float(ans) = cost.eval(&mut ctxt.into()).expect("TODO").coerce_float() {
                            ans.to_f64()
                        } else {
                            //TODO: cooked
                            0.0
                        }
                    },
                    _ => 0.0 //TODO(Dhruv) handle 
                }
            })
        );

        println!("result {jacob:?}");
        jacob
    }
}

impl CostFunction for Problem {
    type Param = DVector<f64>;

    type Output = f64;

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, argmin_math::Error> {
        let cost = self.apply(param)?.sum().abs();
        println!("Have cost {cost:?}");
        Ok(cost)
    }
}

impl Gradient for Problem {
    type Param = DVector<f64>;

    type Gradient = DVector<f64>;

    fn gradient(&self, param: &Self::Param) -> Result<Self::Gradient, argmin_math::Error> {
        let mut grad = self.jacobian(param)?.row_sum().transpose();
        if self.cost(&param)? < 0. {
            grad = -grad;
        }

        println!("Have gradient {grad:?}");
        Ok(grad)
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

    pub fn get_parameter(&self, p: ParameterId) -> Option<&Parameter> {
        self.parameters.get(p.0)
    }

    pub fn add_point(&mut self, x: Parameter, y: Parameter, z: Parameter) -> PointId {
        let x = self.add_parameter(x);
        let y = self.add_parameter(y);
        let z = self.add_parameter(z);

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
        let radius = self.add_parameter(radius);
        self.circles.push(Circle {
            origin,
            radius
        });
        return CircleId(self.circles.len()-1);
    }
}

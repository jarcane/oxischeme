// Copyright 2014 Nick Fitzgerald
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Scheme value implementation.

use std::default::{Default};

use environment::{EnvironmentPtr, RootedEnvironmentPtr};
use heap::{ArenaPtr, GcThing, Heap, IterGcThing, Rooted, RootedStringPtr,
           StringPtr, ToGcThing, Trace};
use context::{Context};

/// A cons cell is a pair of `car` and `cdr` values. A list is one or more cons
/// cells, daisy chained together via the `cdr`. A list is "proper" if the last
/// `cdr` is `Value::EmptyList`, or the scheme value `()`. Otherwise, it is
/// "improper".
#[deriving(Copy, Eq, Hash, PartialEq)]
pub struct Cons {
    car: Value,
    cdr: Value,
}

impl Default for Cons {
    /// Do not use this method, instead allocate cons cells on the heap with
    /// `Heap::allocate_cons` and get back a `ConsPtr`.
    fn default() -> Cons {
        Cons {
            car: Value::EmptyList,
            cdr: Value::EmptyList,
        }
    }
}

impl Cons {
    /// Get the car of this cons cell.
    pub fn car(&self, heap: &mut Heap) -> RootedValue {
        Rooted::new(heap, self.car)
    }

    /// Get the cdr of this cons cell.
    pub fn cdr(&self, heap: &mut Heap) -> RootedValue {
        Rooted::new(heap, self.cdr)
    }

    /// Set the car of this cons cell.
    pub fn set_car(&mut self, car: &RootedValue) {
        self.car = **car;
    }

    /// Set the cdr of this cons cell.
    pub fn set_cdr(&mut self, cdr: &RootedValue) {
        self.cdr = **cdr;
    }
}

impl Trace for Cons {
    fn trace(&self) -> IterGcThing {
        let mut results = vec!();

        if let Some(car) = self.car.to_gc_thing() {
            results.push(car);
        }

        if let Some(cdr) = self.cdr.to_gc_thing() {
            results.push(cdr);
        }

        results.into_iter()
    }
}

/// A pointer to a cons cell on the heap.
pub type ConsPtr = ArenaPtr<Cons>;

impl ToGcThing for ConsPtr {
    fn to_gc_thing(&self) -> Option<GcThing> {
        Some(GcThing::from_cons_ptr(*self))
    }
}

/// A rooted pointer to a cons cell on the heap.
pub type RootedConsPtr = Rooted<ConsPtr>;

/// Procedures are represented by their parameter list, body, and a pointer to
/// their definition environment.
#[deriving(Copy, Hash)]
pub struct Procedure {
    params: Value,
    body: Value,
    env: EnvironmentPtr,
}

impl Procedure {
    /// Get this procedure's parameters.
    pub fn get_params(&self, heap: &mut Heap) -> RootedValue {
        Rooted::new(heap, self.params)
    }

    /// Get this procedure's body.
    pub fn get_body(&self, heap: &mut Heap) -> RootedValue {
        Rooted::new(heap, self.body)
    }

    /// Get this procedure's environment.
    pub fn get_env(&self, heap: &mut Heap) -> RootedEnvironmentPtr {
        Rooted::new(heap, self.env)
    }

    /// Set this procedure's parameters.
    pub fn set_params(&mut self, params: &RootedValue) {
        self.params = **params;
    }

    /// Set this procedure's body.
    pub fn set_body(&mut self, body: &RootedValue) {
        self.body = **body;
    }

    /// Set this procedure's environment.
    pub fn set_env(&mut self, env: &RootedEnvironmentPtr) {
        self.env = **env;
    }
}

impl Default for Procedure {
    /// Do not use this method, instead allocate procedures on the heap with
    /// `Heap::allocate_procedure` and get back a `ProcedurePtr`.
    fn default() -> Procedure {
        Procedure {
            params: Value::EmptyList,
            body: Value::EmptyList,
            env: ArenaPtr::null(),
        }
    }
}

impl Trace for Procedure {
    fn trace(&self) -> IterGcThing {
        let mut results = vec!();

        if let Some(params) = self.params.to_gc_thing() {
            results.push(params);
        }

        if let Some(body) = self.body.to_gc_thing() {
            results.push(body);
        }

        results.push(GcThing::from_environment_ptr(self.env));
        results.into_iter()
    }
}

/// A pointer to a `Procedure` on the heap.
pub type ProcedurePtr = ArenaPtr<Procedure>;
impl ToGcThing for ProcedurePtr {
    fn to_gc_thing(&self) -> Option<GcThing> {
        Some(GcThing::from_procedure_ptr(*self))
    }
}

/// A rooted pointer to a `Procedure` on the heap.
pub type RootedProcedurePtr = Rooted<ProcedurePtr>;

/// `Value` represents a scheme value of any type.
///
/// Note that `Eq` and `PartialEq` are object identity, not structural
/// comparison, same as with [`ArenaPtr`](struct.ArenaPtr.html).
#[deriving(Copy, Eq, Hash, PartialEq, Show)]
pub enum Value {
    /// The empty list: `()`.
    EmptyList,

    /// The scheme pair type is a pointer to a GC-managed `Cons` cell.
    Pair(ConsPtr),

    /// The scheme string type is a pointer to a GC-managed `String`.
    String(StringPtr),

    /// Scheme symbols are also implemented as a pointer to a GC-managed
    /// `String`.
    Symbol(StringPtr),

    /// Scheme integers are represented as 64 bit integers.
    Integer(i64),

    /// Scheme booleans are represented with `bool`.
    Boolean(bool),

    /// Scheme characters are `char`s.
    Character(char),

    /// A Scheme procedure is a pointer to a GC-managed `Procedure`.
    Procedure(ProcedurePtr),
}

/// # `Value` Constructors
impl Value {
    /// Create a new integer value.
    pub fn new_integer(i: i64) -> Value {
        Value::Integer(i)
    }

    /// Create a new boolean value.
    pub fn new_boolean(b: bool) -> Value {
        Value::Boolean(b)
    }

    /// Create a new character value.
    pub fn new_character(c: char) -> Value {
        Value::Character(c)
    }

    /// Create a new cons pair value with the given car and cdr.
    pub fn new_pair(heap: &mut Heap,
                    car: &RootedValue,
                    cdr: &RootedValue) -> RootedValue {
        let mut cons = heap.allocate_cons();
        cons.set_car(car);
        cons.set_cdr(cdr);
        Rooted::new(heap, Value::Pair(*cons))
    }

    /// Create a new procedure with the given parameter list and body.
    pub fn new_procedure(heap: &mut Heap,
                         params: &RootedValue,
                         body: &RootedValue,
                         env: &RootedEnvironmentPtr) -> RootedValue {
        let mut procedure = heap.allocate_procedure();
        procedure.set_params(params);
        procedure.set_body(body);
        procedure.set_env(env);
        Rooted::new(heap, Value::Procedure(*procedure))
    }

    /// Create a new string value with the given string.
    pub fn new_string(heap: &mut Heap, str: String) -> RootedValue {
        let mut value = heap.allocate_string();
        value.clear();
        value.push_str(str.as_slice());
        Rooted::new(heap, Value::String(*value))
    }

    /// Create a new symbol value with the given string.
    pub fn new_symbol(ctx: &Context, str: RootedStringPtr) -> RootedValue {
        Rooted::new(ctx.heap(), Value::Symbol(*str))
    }
}

/// # `Value` Methods
impl Value {
    /// Assuming this value is a cons pair, get its car value. Otherwise, return
    /// `None`.
    pub fn car(&self, heap: &mut Heap) -> Option<RootedValue> {
        match *self {
            Value::Pair(ref cons) => Some(cons.car(heap)),
            _                     => None,
        }
    }

    /// Assuming this value is a cons pair, get its cdr value. Otherwise, return
    /// `None`.
    pub fn cdr(&self, heap: &mut Heap) -> Option<RootedValue> {
        match *self {
            Value::Pair(ref cons) => Some(cons.cdr(heap)),
            _                     => None,
        }
    }

    /// Return true if this value is a pair, false otherwise.
    pub fn is_pair(&self) -> bool {
        match *self {
            Value::Pair(_) => true,
            _              => false,
        }
    }

    /// Return true if this value is an atom, false otherwise.
    pub fn is_atom(&self) -> bool {
        !self.is_pair()
    }

    /// Coerce this symbol value to a `StringPtr` to the symbol's string name.
    pub fn to_symbol(&self, heap: &mut Heap) -> Option<RootedStringPtr> {
        match *self {
            Value::Symbol(sym) => Some(Rooted::new(heap, sym)),
            _                  => None,
        }
    }

    /// Coerce this pair value to a `ConsPtr` to the cons cell this pair is
    /// referring to.
    pub fn to_pair(&self, heap: &mut Heap) -> Option<RootedConsPtr> {
        match *self {
            Value::Pair(cons) => Some(Rooted::new(heap, cons)),
            _                 => None,
        }
    }

    /// Coerce this procedure value to a `ProcedurePtr` to the `Procedure` this
    /// value is referring to.
    pub fn to_procedure(&self, heap: &mut Heap) -> Option<RootedProcedurePtr> {
        match *self {
            Value::Procedure(p) => Some(Rooted::new(heap, p)),
            _                   => None,
        }
    }

    /// Assuming that this value is a proper list, get the length of the list.
    pub fn len(&self) -> Result<u64, ()> {
        match *self {
            Value::EmptyList => Ok(0),
            Value::Pair(p)   => {
                let cdr_len = try!(p.cdr.len());
                Ok(cdr_len + 1)
            },
            _                => Err(()),
        }
    }
}

impl ToGcThing for Value {
    fn to_gc_thing(&self) -> Option<GcThing> {
        match *self {
            Value::String(str)  => Some(GcThing::from_string_ptr(str)),
            Value::Symbol(sym)  => Some(GcThing::from_string_ptr(sym)),
            Value::Pair(cons)   => Some(GcThing::from_cons_ptr(cons)),
            Value::Procedure(p) => Some(GcThing::from_procedure_ptr(p)),
            _                   => None,
        }
    }
}

pub type RootedValue = Rooted<Value>;

/// Either a Scheme `RootedValue`, or a `String` containing an error message.
pub type SchemeResult = Result<RootedValue, String>;

/// A helper utility to create a cons list from the given values.
pub fn list(ctx: &mut Context, values: &[RootedValue]) -> RootedValue {
    list_helper(ctx, &mut values.iter())
}

fn list_helper<'a, T: Iterator<&'a RootedValue>>(ctx: &mut Context,
                                                 values: &mut T) -> RootedValue {
    match values.next() {
        None      => Rooted::new(ctx.heap(), Value::EmptyList),
        Some(car) => {
            let rest = list_helper(ctx, values);
            Value::new_pair(ctx.heap(), car, &rest)
        },
    }
}

/// ## The 28 car/cdr compositions.
impl Cons {
    pub fn cddr(&self, ctx: &Context) -> SchemeResult {
        self.cdr.cdr(ctx.heap()).ok_or("bad cddr".to_string())
    }

    pub fn cdddr(&self, ctx: &Context) -> SchemeResult {
        let cddr = try!(self.cddr(ctx));
        cddr.cdr(ctx.heap()).ok_or("bad cdddr".to_string())
    }

    // TODO FITZGEN: cddddr

    pub fn cadr(&self, ctx: &Context) -> SchemeResult {
        self.cdr.car(ctx.heap()).ok_or("bad cadr".to_string())
    }

    pub fn caddr(&self, ctx: &Context) -> SchemeResult {
        let cddr = try!(self.cddr(ctx));
        cddr.car(ctx.heap()).ok_or("bad caddr".to_string())
    }

    pub fn cadddr(&self, ctx: &Context) -> SchemeResult {
        let cdddr = try!(self.cdddr(ctx));
        cdddr.car(ctx.heap()).ok_or("bad caddr".to_string())
    }

    // TODO FITZGEN ...
}


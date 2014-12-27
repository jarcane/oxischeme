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

use heap::{ArenaPtr, ConsPtr, EnvironmentPtr, GcThing, Heap, IterGcThing,
           ProcedurePtr, StringPtr, Trace};
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
    pub fn car(&self) -> Value {
        self.car
    }

    /// Get the cdr of this cons cell.
    pub fn cdr(&self) -> Value {
        self.cdr
    }

    /// Set the car of this cons cell.
    pub fn set_car(&mut self, car: Value) {
        self.car = car;
    }

    /// Set the cdr of this cons cell.
    pub fn set_cdr(&mut self, cdr: Value) {
        self.cdr = cdr;
    }
}

/// TODO FITZGEN
impl Trace for Cons {
    /// TODO FITZGEN
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
    pub fn get_params(&self) -> Value {
        self.params
    }

    /// Get this procedure's body.
    pub fn get_body(&self) -> Value {
        self.body
    }

    /// Get this procedure's environment.
    pub fn get_env(&self) -> EnvironmentPtr {
        self.env
    }

    /// Set this procedure's parameters.
    pub fn set_params(&mut self, params: Value) {
        self.params = params;
    }

    /// Set this procedure's body.
    pub fn set_body(&mut self, body: Value) {
        self.body = body;
    }

    /// Set this procedure's environment.
    pub fn set_env(&mut self, env: EnvironmentPtr) {
        self.env = env;
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
    /// TODO FITZGEN
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
    pub fn new_pair(heap: &mut Heap, car: Value, cdr: Value) -> Value {
        let mut cons = heap.allocate_cons();
        cons.set_car(car);
        cons.set_cdr(cdr);
        Value::Pair(cons)
    }

    /// Create a new procedure with the given parameter list and body.
    pub fn new_procedure(heap: &mut Heap,
                         params: Value,
                         body: Value,
                         env: EnvironmentPtr) -> Value {
        let mut procedure = heap.allocate_procedure();
        procedure.set_params(params);
        procedure.set_body(body);
        procedure.set_env(env);
        Value::Procedure(procedure)
    }

    /// Create a new string value with the given string.
    pub fn new_string(heap: &mut Heap, str: String) -> Value {
        let mut value = heap.allocate_string();
        value.clear();
        value.push_str(str.as_slice());
        Value::String(value)
    }

    /// Create a new symbol value with the given string.
    pub fn new_symbol(str: StringPtr) -> Value {
        Value::Symbol(str)
    }
}

/// # `Value` Methods
impl Value {
    /// Assuming this value is a cons pair, get its car value. Otherwise, return
    /// `None`.
    pub fn car(&self) -> Option<Value> {
        match *self {
            Value::Pair(ref cons) => Some(cons.car()),
            _                     => None,
        }
    }

    /// Assuming this value is a cons pair, get its cdr value. Otherwise, return
    /// `None`.
    pub fn cdr(&self) -> Option<Value> {
        match *self {
            Value::Pair(ref cons) => Some(cons.cdr()),
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
    pub fn to_symbol(&self) -> Option<StringPtr> {
        match *self {
            Value::Symbol(sym) => Some(sym),
            _                  => None,
        }
    }

    /// Coerce this pair value to a `ConsPtr` to the cons cell this pair is
    /// referring to.
    pub fn to_pair(&self) -> Option<ConsPtr> {
        match *self {
            Value::Pair(cons) => Some(cons),
            _                 => None,
        }
    }

    /// Coerce this procedure value to a `ProcedurePtr` to the `Procedure` this
    /// value is referring to.
    pub fn to_procedure(&self) -> Option<ProcedurePtr> {
        match *self {
            Value::Procedure(p) => Some(p),
            _                   => None,
        }
    }

    /// Assuming that this value is a proper list, get the length of the list.
    pub fn len(&self) -> Result<u64, ()> {
        match *self {
            Value::EmptyList => Ok(0),
            Value::Pair(p)   => {
                let cdr_len = try!(p.cdr().len());
                Ok(cdr_len + 1)
            },
            _                => Err(()),
        }
    }

    /// TODO FITZGEN
    pub fn to_gc_thing(&self) -> Option<GcThing> {
        match *self {
            Value::String(str)  => Some(GcThing::from_string_ptr(str)),
            Value::Symbol(sym)  => Some(GcThing::from_string_ptr(sym)),
            Value::Pair(cons)   => Some(GcThing::from_cons_ptr(cons)),
            Value::Procedure(p) => Some(GcThing::from_procedure_ptr(p)),
            _                   => None,
        }
    }
}

/// Either a Scheme `Value`, or a `String` containing an error message.
pub type SchemeResult = Result<Value, String>;

/// A helper utility to create a cons list from the given values.
pub fn list(ctx: &mut Context, values: &[Value]) -> Value {
    list_helper(ctx, &mut values.iter())
}

fn list_helper<'a, T: Iterator<&'a Value>>(ctx: &mut Context,
                                           values: &mut T) -> Value {
    match values.next() {
        None      => Value::EmptyList,
        Some(car) => {
            let cdr = list_helper(ctx, values);
            Value::new_pair(ctx.heap(), *car, cdr)
        },
    }
}

/// ## The 28 car/cdr compositions.
impl Cons {
    pub fn cddr(&self) -> SchemeResult {
        self.cdr.cdr().ok_or("bad cddr".to_string())
    }

    pub fn cdddr(&self) -> SchemeResult {
        let cddr = try!(self.cddr());
        cddr.cdr().ok_or("bad cdddr".to_string())
    }

    // TODO FITZGEN: cddddr

    pub fn cadr(&self) -> SchemeResult {
        self.cdr.car().ok_or("bad cadr".to_string())
    }

    pub fn caddr(&self) -> SchemeResult {
        let cddr = try!(self.cddr());
        cddr.car().ok_or("bad caddr".to_string())
    }

    pub fn cadddr(&self) -> SchemeResult {
        let cdddr = try!(self.cdddr());
        cdddr.car().ok_or("bad caddr".to_string())
    }

    // TODO FITZGEN ...
}


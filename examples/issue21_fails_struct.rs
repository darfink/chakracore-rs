extern crate chakracore;

use std::io::prelude::*;
use std::fs::File;

pub struct Js<'a> {
    guard: &'a chakracore::context::ContextGuard<'a>,
}

impl<'a> Js<'a> {
    pub fn new(guard: &'a chakracore::context::ContextGuard<'a>) -> Js<'a> {
        // load typescript.js
        let js = "./node_modules/typescript/lib/typescript.js";
        let mut file = File::open(js).expect("unable to open the file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("unable to read the file");
        chakracore::script::eval(&guard, &contents).expect("invalid JavaScript code");
        Js { guard: guard }
    }
}

fn main() {
    let runtime = chakracore::Runtime::new().unwrap();
    let context = chakracore::Context::new(&runtime).unwrap();
    let guard = context.make_current().unwrap();

    let js = Js::new(&guard);

    // getting the ts.version works
    let v = chakracore::script::eval(js.guard, "ts.version").unwrap().to_string(&guard);
    println!("v: {:?}", v);

    let ts = chakracore::script::eval(js.guard, "ts").unwrap().into_object().unwrap();

    // it appears to get the function
    let function = ts.get(js.guard, &chakracore::Property::new(js.guard, "createNode")).into_function().unwrap();
    println!("got function createNode");

    // call createNode, this fails:
    let rv = function.call_with_this(js.guard, &ts, &[
        &chakracore::value::Number::new(js.guard, 3).into(),
        &chakracore::value::Number::new(js.guard, -1).into(),
        &chakracore::value::Number::new(js.guard, -1).into(),
    ]);
    println!("rv: {:?}", rv);
}

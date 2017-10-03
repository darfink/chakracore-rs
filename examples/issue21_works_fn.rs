extern crate chakracore;

use std::io::prelude::*;
use std::fs::File;

fn main() {
    let runtime = chakracore::Runtime::new().unwrap();
    let context = chakracore::Context::new(&runtime).unwrap();
    let guard = context.make_current().unwrap();

    // load typescript.js
    let js = "./node_modules/typescript/lib/typescript.js";
    let mut file = File::open(js).expect("unable to open the file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("unable to read the file");
    chakracore::script::eval(&guard, &contents).expect("invalid JavaScript code");

    call_create_node(&guard);
    call_create_node(&guard);
}

fn call_create_node<'a>(guard: &'a chakracore::context::ContextGuard<'a>){
    // get ts variable
    let ts = guard.global().get(guard, &chakracore::Property::new(guard, "ts")).into_object().unwrap();

    // call createNode
    let function = ts.get(guard, &chakracore::Property::new(guard, "createNode")).into_function().unwrap();
    println!("got function createNode");
    let rv = function.call_with_this(guard, &ts, &[
        &chakracore::value::Number::new(guard, 3).into(),
        &chakracore::value::Number::new(guard, -1).into(),
        &chakracore::value::Number::new(guard, -1).into(),
    ]);
    println!("rv: {:?}", rv);
    let node = rv.unwrap().into_object().unwrap();

    // verify that the node kind is 3
    let kind = node.get(&guard, &chakracore::Property::new(guard, "kind"));
    println!("kind: {:?}", kind);
    println!("kind: {:?}", kind.into_number().unwrap().value());
}

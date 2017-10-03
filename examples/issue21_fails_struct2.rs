#![allow(non_snake_case)]

extern crate chakracore;
use std::io::prelude::*;
use std::fs::File;

pub struct Js<'a> {
    guard: &'a chakracore::context::ContextGuard<'a>,
}

impl<'a> Js<'a> {
    pub fn read_js() -> String {
        let js = "./node_modules/typescript/lib/typescript.js";
        let mut file = File::open(js).expect("unable to open the file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("unable to read the file");
        contents
    }

    pub fn new(guard: &'a chakracore::context::ContextGuard<'a>) -> Js<'a> {
        let js = Js::read_js();
        chakracore::script::eval(guard, &js).expect("invalid JavaScript code");
        Js { guard: guard }
    }

    pub fn createNode(&self, kind: i32) -> Node {
        // get ts variable
        let ts = self.guard.global().get(self.guard, &chakracore::Property::new(self.guard, "ts")).into_object().unwrap();

        // call createNode
        let function = ts.get(self.guard, &chakracore::Property::new(self.guard, "createNode")).into_function().unwrap();
        let rv = function.call_with_this(self.guard, &ts, &[
            &chakracore::value::Number::new(self.guard, kind).into(),
            &chakracore::value::Number::new(self.guard, -1).into(),
            &chakracore::value::Number::new(self.guard, -1).into(),
        ]);
        let object = rv.unwrap().into_object().unwrap();
        Node { object: object }
    }
}

pub struct Node {
    object: chakracore::value::Object,
}

impl Node {
    pub fn kind(&self, guard: &chakracore::context::ContextGuard) -> i32 {
        let kind = self.object.get(&guard, &chakracore::Property::new(&guard, "kind"));
        kind.into_number().unwrap().value()
    }
}

fn main() {
    let runtime = chakracore::Runtime::new().unwrap();
    let context = chakracore::Context::new(&runtime).unwrap();
    let guard = context.make_current().unwrap();

    let js = Js::new(&guard);

    let node = js.createNode(3);
    println!("kind: {:?}", node.kind(&guard));
}

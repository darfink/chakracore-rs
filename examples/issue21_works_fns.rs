#![allow(non_snake_case)]

extern crate chakracore;

pub mod ts {
    extern crate chakracore;
    use std::fs::File;
    use std::io::prelude::*;

    pub fn read_js() -> String {
        let js = "./node_modules/typescript/lib/typescript.js";
        let mut file = File::open(js).expect("unable to open the file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("unable to read the file");
        contents
    }

    pub fn new_context() -> (chakracore::Runtime, chakracore::Context) {
        let runtime = chakracore::Runtime::new().unwrap();
        let context = chakracore::Context::new(&runtime).unwrap();
        (runtime, context)
    }

    pub fn eval_js(guard: &chakracore::context::ContextGuard, js: &str) {
        chakracore::script::eval(guard, js).expect("invalid JavaScript code");
    }

    pub fn createNode(guard: &chakracore::context::ContextGuard, kind: i32) -> Node {
        // get ts variable
        let ts = guard.global().get(guard, &chakracore::Property::new(guard, "ts")).into_object().unwrap();

        // call createNode
        let function = ts.get(guard, &chakracore::Property::new(guard, "createNode")).into_function().unwrap();
        let rv = function.call_with_this(guard, &ts, &[
            &chakracore::value::Number::new(guard, kind).into(),
            &chakracore::value::Number::new(guard, -1).into(),
            &chakracore::value::Number::new(guard, -1).into(),
        ]);
        let object = rv.unwrap().into_object().unwrap();
        Node { object: object }
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
}

fn main() {
    let (_runtime, context) = ts::new_context();
    let guard = context.make_current().unwrap();
    
    let js = ts::read_js();
    ts::eval_js(&guard, &js);
    
    let node = ts::createNode(&guard, 3);
    println!("kind: {:?}", node.kind(&guard));
}
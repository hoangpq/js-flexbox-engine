extern crate mini_v8;
extern crate yoga;

use yoga::FlexDirection;
use yoga::Node;
use yoga::*;

use mini_v8::{Error as MV8Error, Invocation, MiniV8, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;

#[allow(dead_code)]
fn read_file_to_buf(filename: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(filename)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}

fn read_file(filename: &str) -> std::io::Result<String> {
    let mut file = File::open(filename)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    Ok(data)
}

fn write_to_file(data: &str) -> std::io::Result<()> {
    let mut file = File::create("layout.html")?;
    file.write_all(data.as_ref())?;
    Ok(())
}

fn load_bundle(mv8: &MiniV8, path: &str) {
    if let Ok(output) = Command::new("npm")
        .arg("run")
        .arg("build")
        .arg(path)
        .output()
    {
        let script = String::from_utf8_lossy(&output.stdout);
        let script = script
            .lines()
            .enumerate()
            .skip_while(|&(index, _)| index < 3)
            .map(|(_, c)| c)
            .collect::<Vec<&str>>()
            .join("\n");

        if let Ok(_val) = mv8.eval::<Value>(&script) {
            let script = r#"
                const root = render();
                calculateLayout(root.node);
                writeData(root.render());
            "#;
            let _: Result<Value, MV8Error> = mv8.eval(script);
        }
    }
}

fn main() {
    thread_local! {
        static ROOT: RefCell<u64> = RefCell::new(0u64);
        static LAYOUT: RefCell<HashMap<u64, Node>> = RefCell::new(HashMap::new());
    }

    let mv8 = MiniV8::new();

    let create_node = mv8.create_function(move |inv: Invocation| {
        let (child_nodes, style): (Vec<u64>, HashMap<String, Value>) = inv.args.into(inv.mv8)?;

        let flex_shrink = match style.get("flexShrink") {
            Some(val) => val.as_number().unwrap() as f32,
            None => 1f32,
        };

        let flex_grow = match style.get("flexGrow") {
            Some(val) => val.as_number().unwrap() as f32,
            None => 0f32,
        };

        let flex_direction = match style.get("flexDirection") {
            Some(val) => match &*val.as_string().unwrap().to_string() {
                "column" => FlexDirection::Column,
                _ => FlexDirection::Row,
            },
            None => FlexDirection::Row,
        };

        dbg!(flex_direction);

        let width = match style.get("width") {
            Some(val) => {
                let val = val.as_number().unwrap_or(0f64) as f32;
                StyleUnit::Point(val.into())
            }
            None => StyleUnit::Auto,
        };

        let height = match style.get("height") {
            Some(val) => {
                let val = val.as_number().unwrap_or(0f64) as f32;
                StyleUnit::Point(val.into())
            }
            None => StyleUnit::Auto,
        };

        let mut node = Node::new();
        node.set_flex_direction(flex_direction);
        node.set_flex_shrink(flex_shrink);
        node.set_flex_grow(flex_grow);
        node.set_width(width);
        node.set_height(height);

        Ok(LAYOUT.with(|layout| {
            let mut layout = layout.borrow_mut();

            child_nodes
                .clone()
                .iter()
                .enumerate()
                .for_each(|(index, child_ptr)| {
                    if let Some(child) = layout.get_mut(child_ptr) {
                        node.insert_child(child, index as u32);
                    }
                });

            let node_ptr = Box::into_raw(Box::new(&node)) as u64;
            layout.insert(node_ptr, node);

            ROOT.with(|root| {
                // update root node
                *root.borrow_mut() = node_ptr;
                node_ptr
            })
        }))
    });

    let calculate_layout = mv8.create_function(|_inv: Invocation| {
        ROOT.with(|root| {
            let node = *root.borrow();

            LAYOUT.with(|layout| {
                let mut layout = layout.borrow_mut();

                if let Some(node) = layout.get_mut(&node) {
                    node.calculate_layout(
                        node.get_layout_width(),
                        node.get_layout_height(),
                        yoga::Direction::LTR,
                    );
                }
            });
        });

        Ok(())
    });

    let get_layout = mv8.create_function(|inv: Invocation| {
        let (node,): (u64,) = inv.args.into(inv.mv8)?;

        Ok(LAYOUT.with(|layout| {
            let mut layout = layout.borrow_mut();
            let mut style: HashMap<&str, f32> = HashMap::new();

            if let Some(node) = layout.get_mut(&node) {
                let layout = node.get_layout();
                dbg!(&layout);

                style.insert("top", layout.top());
                style.insert("left", layout.left());
                style.insert("width", layout.width());
                style.insert("height", layout.height());
            }

            style
        }))
    });

    let write_data = mv8.create_function(|inv: Invocation| {
        let (data,): (String,) = inv.args.into(inv.mv8)?;
        write_to_file(&data).unwrap();
        Ok(())
    });

    let print = mv8.create_function(|inv: Invocation| {
        let (msg,): (String,) = inv.args.into(inv.mv8)?;
        dbg!(msg);

        Ok(())
    });

    let global = mv8.global();

    global.set("print", print.clone()).unwrap();
    global
        .set("calculateLayout", calculate_layout.clone())
        .unwrap();

    global.set("createNode", create_node.clone()).unwrap();
    global.set("getLayout", get_layout.clone()).unwrap();
    global.set("writeData", write_data.clone()).unwrap();

    let engine_script = read_file("src/engine.js").unwrap();
    let _: Result<Value, MV8Error> = mv8.eval(&engine_script);

    load_bundle(&mv8, "jsx/layout.js");
}

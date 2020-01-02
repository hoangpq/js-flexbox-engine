extern crate mini_v8;
extern crate yoga;

use yoga::Node;
use yoga::*;
use yoga::{FlexDirection, Wrap};

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

        let mut node = Node::new();

        if let Some(val) = style.get("flexShrink") {
            let val = val.as_number().expect("flexShrink must be number") as f32;
            node.set_flex_shrink(val);
        }

        if let Some(val) = style.get("flexGrow") {
            let val = val.as_number().expect("flexGrow must be number") as f32;
            node.set_flex_grow(val);
        }

        if let Some(val) = style.get("flexDirection") {
            let val = &*val
                .as_string()
                .expect("flexDirection must be string")
                .to_string();
            let val = match val {
                "column" => FlexDirection::Column,
                _ => FlexDirection::Row,
            };
            node.set_flex_direction(val);
        }

        if let Some(val) = style.get("flexWrap") {
            let val = &*val
                .as_string()
                .expect("flexWrap must be string")
                .to_string();
            let val = match val {
                "wrap" => Wrap::Wrap,
                _ => Wrap::NoWrap,
            };
            node.set_flex_wrap(val);
        }

        if let Some(val) = style.get("width") {
            let val = val.as_number().expect("width must be number") as f32;
            node.set_width(StyleUnit::Point(val.into()));
        }

        if let Some(val) = style.get("height") {
            let val = val.as_number().expect("height must be number") as f32;
            node.set_height(StyleUnit::Point(val.into()));
        }

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

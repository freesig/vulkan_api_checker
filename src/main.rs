use std::io::stdin;
use std::io::{BufRead, Stdin};
use std::collections::{HashMap, HashSet};
use regex::Regex;

enum Transition {
    Empty,
    Old(String),
    Both(LayoutTransition),
}

#[derive(Default)]
struct LayoutTransition {
    old_layout: String,
    new_layout: String,
}

#[derive(Default)]
struct Layout {
    src: Option<HashSet<String>>,
    dst: Option<HashSet<String>>,
}

type Rules = HashMap<String, Layout>;

type ImageLayout = HashMap<String, String>;

// fn name to associated image search
type AssociatedImage = HashMap<String, Regex>;

#[derive(Debug)]
enum LayoutErr {
    Src,
    Dst,
}

fn load_rules() -> Rules {
    let mut rules = HashMap::new();
    let dst = ["VK_IMAGE_LAYOUT_GENERAL", "VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL", "VK_IMAGE_LAYOUT_SHARED_PRESENT_KHR"];
    rules.insert("CmdCopyBufferToImage".to_string(), Layout{src: None, dst: Some(dst.into_iter().map(|s| s.to_string()).collect()) });
    rules
}

fn check_layout(layout: &Layout, current_layout: &String) -> Result<(), LayoutErr> {
    //TODO check image ptr is the same. Probably need
    //to store current layouts in a hash from the ptr to the layout
    layout.src
        .as_ref()
        .filter(|src| src.contains(current_layout))
        .map(|_|())
        .ok_or(LayoutErr::Src)?;
    layout.dst
        .as_ref()
        .filter(|dst| dst.contains(current_layout))
        .map(|_|())
        .ok_or(LayoutErr::Dst)?;
    Ok(())
}

fn search<T: Target>(input: &mut Stdin, target: T) {
    loop {
        let buf = input.lock();
        buf.lines()
            .and_then(Result::ok)
            .map(|l| target.check(&l))
}

fn main() {
    let rules = load_rules();
    let vk_function_re = Regex::new(r"^vk(?P<fn_name>[^\(\s]+)\(").expect("failed to create function regex");
    let old_layout_re = Regex::new(r"oldLayout:[^=]+=\s(?P<old_layout>\w+)").expect("failed to create old layout regex");
    let new_layout_re = Regex::new(r"newLayout:[^=]+=\s(?P<new_layout>\w+)").expect("failed to create old layout regex");
    let image_ptr_re = Regex::new(r"image:[^=]+=\s(?P<img_ptr>\w+)").expect("failed to create old layout regex");
    let mut last_fn = String::new();
    let mut current_layout = String::new();
    let mut transition = Transition::Empty;
    let input = stdin();
    loop {
        let buf = input.lock();
        for line in buf.lines() {
            match line {
                Ok(l) => {
                    if let Some(found) = vk_function_re.captures(&l).and_then(|cap| {
                        cap.name("fn_name").map(|fn_name| fn_name.as_str())
                    }) {
                        last_fn = found.to_string();
                        rules.get(&last_fn)
                            .map(|r| {
                                match check_layout(&r, &current_layout) {
                                    Ok(_) => println!("layout OK!"),
                                    Err(_) => println!("Layout wrong!"),
                                }
                            });
                    }
                    transition = match transition {
                        Transition::Empty => {
                            match old_layout_re.captures(&l).and_then(|cap| {
                                cap.name("old_layout")
                            }) {
                                Some(ol) => Transition::Old(ol.as_str().to_owned()),
                                None => Transition::Empty,
                            }
                        },
                        Transition::Old(ol) => {
                            match new_layout_re.captures(&l).and_then(|cap| {
                                cap.name("new_layout")
                            }) {
                                Some(nl) => Transition::Both(LayoutTransition{old_layout: ol, new_layout: nl.as_str().to_owned()}),
                                None => Transition::Empty,
                            }
                        },
                        Transition::Both(t) => {
                            match image_ptr_re.captures(&l).and_then(|cap| {
                                cap.name("img_ptr")
                            }) {
                                Some(ptr) => {
                                    images.insert(ptr.as_str().to_owned(), t.new_layout);
                                    Transition::Empty
                                },
                                None => Transition::Both(t),
                            }
                            //println!("vk{}: from: {}, to: {}", last_fn, t.old_layout, t.new_layout);
                        },
                    }

                },
                Err(ref e) => eprintln!("error: {:?}", e),
            }
        }
    }
}

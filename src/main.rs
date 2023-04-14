extern crate pest;
#[allow(unused_imports)]
#[macro_use]
extern crate pest_derive;
mod parse;

use parse::*;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::{fs, io};
use tera::{Context, Tera};

fn string_prompt(prompt: &str) -> String {
    println!("{prompt}: ");

    let mut component_name = String::new();

    io::stdin()
        .read_line(&mut component_name)
        .expect("Failed to read line");
    let result = component_name.trim().to_string();

    println!("You entered: {}", result);
    result
}

fn confirm_prompt(prompt: &str) -> bool {
    print!("{} (y/N): ", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    input.to_lowercase().starts_with('y')
}

fn main() {
    let component_proto = ComponentServiceProto::from_path(&PathBuf::from("./motor.proto"));

    println!("{:#?}", component_proto);
    let mut tera = match Tera::new("templates/**/*.template") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    tera.register_filter("lower", tera_text_filters::lower_case);
    tera.register_filter("camel", tera_text_filters::camel_case);
    let mut context = Context::new();
    context.insert("component_name", "Motor");
    let res = tera.render("component.hpp.template", &context).unwrap();
    println!("{}", res);
}

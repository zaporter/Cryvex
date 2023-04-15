extern crate pest;
#[allow(unused_imports)]
#[macro_use]
extern crate pest_derive;
mod parse;
mod template;
mod utils;

use parse::*;
use std::path::PathBuf;

pub struct CliOpts {
    prompt_for_struct_names: bool,
    prompt_for_function_names: bool,
    include_name_field: bool,
    include_extra_field: bool,
}

fn main() {
    let component_proto =
        ComponentServiceProto::from_path(&PathBuf::from("./motor.proto")).unwrap();
    let opts = CliOpts {
        prompt_for_struct_names: false,
        prompt_for_function_names: false,
        include_name_field: false,
        include_extra_field: false,
    };

    println!("{:#?}", component_proto);
    template::gen_templates(component_proto, &opts).unwrap();
}

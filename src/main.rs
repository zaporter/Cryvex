extern crate pest;
#[allow(unused_imports)]
#[macro_use]
extern crate pest_derive;
mod parse;
mod template;
mod utils;

use clap::Parser;
use env_logger::Env;

use parse::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about)]
pub struct CliOpts {
    #[arg(short, long, default_value = "./templates")]
    template_folder: PathBuf,
    #[arg(short, long, default_value = "./out")]
    out_folder: PathBuf,
    #[arg(short = 's', long, default_value = "false")]
    dont_prompt_for_struct_names: bool,
    #[arg(short = 'f', long, default_value = "false")]
    dont_prompt_for_function_names: bool,
    #[arg(long, default_value = "false")]
    include_name_field: bool,
    #[arg(long, default_value = "false")]
    include_extra_field: bool,

    proto_path: PathBuf,
}

fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    log::info!("Starting proto parsing");
    let opts = CliOpts::parse();
    let component_proto = ComponentServiceProto::from_path(&opts.proto_path).unwrap();
    log::info!("Finished proto parsing");

    println!("{:#?}", component_proto);
    log::info!("Starting template generation");
    template::gen_templates(component_proto, &opts).unwrap();
    log::info!("Finished template generation");
}

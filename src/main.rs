extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::iterators::Pair;
use pest::{error::Error, iterators::Pairs, Parser};
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::{fs, io};
use tera::{Context, Tera};

#[derive(Clone, Debug)]
pub struct ComponentServiceProto {
    pub name: String,
    pub rpcs: Vec<Rpc>,
    pub messages: Vec<Message>,
}

#[derive(Clone, Debug)]
pub struct Rpc {
    pub comment: Option<String>,
    pub name: String,
    pub request_name: String,
    pub response_name: String,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub comment: Option<String>,
    pub name: String,
    // name -> type
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub comment: Option<String>,
    pub name: String,
    pub proto_type: String,
}

use anyhow::{anyhow, Result};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "proto.pest"]
pub struct ProtoParser;

pub fn from_proto(input: &str) -> Result<ComponentServiceProto> {
    let mut pairs = ProtoParser::parse(Rule::proto, input)?;
    let pairs = pairs.next().unwrap().into_inner();

    let mut service_name = String::new();
    let mut rpcs = Vec::new();
    let mut messages = Vec::new();

    for pair in pairs {
        // if it is topLevelDef, we care, unwrap
        // otherwise ignore.
        let pair = match pair.as_rule() {
            Rule::topLevelDef => pair.into_inner().next().unwrap(),
            _ => continue,
        };

        match pair.as_rule() {
            Rule::service => {
                for service_pair in pair.into_inner() {
                    match service_pair.as_rule() {
                        Rule::serviceName => {
                            service_name = service_pair
                                .into_inner()
                                .next()
                                .unwrap()
                                .as_str()
                                .to_string();
                        }

                        Rule::rpc => {
                            let mut inner = service_pair.into_inner();
                            let name = inner.next().unwrap().as_str().to_string();
                            // let comment = inner
                            //     .next()
                            //     .filter(|p| p.as_rule() == Rule::COMMENT)
                            //     .map(|p| p.as_str().to_string());
                            let request_name = inner.next().unwrap().as_str().to_string();
                            let response_name = inner.next().unwrap().as_str().to_string();
                            rpcs.push(Rpc {
                                comment: None,
                                name,
                                request_name,
                                response_name,
                            });
                        }
                        _ => {}
                    }
                }
            }
            Rule::message => {
                let mut inner = pair.into_inner();
                // let comment = inner
                //     .next()
                //     .filter(|p| p.as_rule() == Rule::COMMENT)
                //     .map(|p| p.as_str().to_string());
                let name = inner.next().unwrap().as_str().to_string();
                let mut fields = Vec::new();

                for field_pair in inner.next().unwrap().into_inner() {
                    if field_pair.as_rule() == Rule::field {
                        let mut field_inner = field_pair.into_inner();
                        // multiplicity field
                        field_inner.next().unwrap();
                        // let field_comment = field_inner
                        //     .next()
                        //     .filter(|p| p.as_rule() == Rule::COMMENT)
                        //     .map(|p| p.as_str().to_string());
                        let proto_type = field_inner.next().unwrap().as_str().to_string();
                        let field_name = field_inner.next().unwrap().as_str().to_string();
                        fields.push(Field {
                            comment: None,
                            name: field_name,
                            proto_type,
                        });
                    }
                }

                messages.push(Message {
                    comment: None,
                    name,
                    fields,
                });
            }
            _ => {}
        }
    }

    if service_name.is_empty() {
        return Err(anyhow!("Service name not found"));
    }

    Ok(ComponentServiceProto {
        name: service_name,
        rpcs,
        messages,
    })
}
impl ComponentServiceProto {
    pub fn from_path(path: &PathBuf) -> anyhow::Result<Self> {
        let proto_string = fs::read_to_string(path)?;
        return from_proto(&proto_string);
    }
}

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

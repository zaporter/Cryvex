use std::{collections::HashMap, path::PathBuf};

use crate::{parse::*, utils, CliOpts};
use anyhow::{anyhow, Context};
use convert_case::Casing;
use serde::Serialize;
use tera::Tera;

const VOID_T: &str = "void";
const FROM_PROTO_T: &str = "from_proto";
const TO_PROTO_T: &str = "to_proto";
const NAME_PARAM: &str = "name";
const EXTRA_PARAM: &str = "extra";

pub fn gen_templates(proto: ComponentServiceProto, opts: &CliOpts) -> anyhow::Result<()> {
    let mut tera = match Tera::new("templates/**/*.template") {
        Ok(t) => t,
        Err(e) => {
            log::error!("Template parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    tera.register_filter("camel", tera_text_filters::camel_case);

    let template = TemplateInput::from_proto(&proto, opts)?;
    let context = tera::Context::from_serialize(&template)?;

    for template_name in tera.get_template_names() {
        // the string 'component' is magic and is replaced by the name of the component
        // mock_component.cpp.template -> ./out/mock_motor.cpp
        let out_name = template_name.trim_end_matches(".template");
        let out_name = out_name.replace("component", &template.component.name);
        let out_name = out_name.to_case(convert_case::Case::Snake);
        let out_path = PathBuf::from(out_name);
        let out_path = opts.out_folder.join(out_path);
        let prefix = &out_path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();
        let result = tera.render(template_name, &context)?;
        std::fs::write(out_path.clone(), result)?;

        println!("Writing to: {}", &out_path.to_string_lossy())
    }
    Ok(())
}

#[derive(Serialize, Clone, Debug)]
struct TemplateInput {
    pub component: ComponentDec,
}

#[derive(Serialize, Clone, Debug)]
struct ComponentDec {
    name: String,
    from_proto_fns: Vec<FnDec>,
    to_proto_fns: Vec<FnDec>,
    member_fns: Vec<FnDec>,
    structs: Vec<StructDec>,
    enums: Vec<Enum>, // from parse.rs
    rpcs: Vec<RpcDec>,
}

#[derive(Serialize, Clone, Debug)]
struct RpcDec {
    comment: Option<String>,
    rpc_name: String,  // GetProperties
    func_name: String, // get_properties
    req_t: String,     // GetPropertiesRequest
    resp_t: String,    // GetPropertiesResponse
}

#[derive(Serialize, Clone, Debug)]
struct StructDec {
    comment: Option<String>,
    name: String,
    members: Vec<Variable>,
}

#[derive(Serialize, Clone, Debug)]
struct FnDec {
    comment: Option<String>,
    name: String,
    return_t: String,
    args: Vec<Variable>,
}

#[derive(Serialize, Clone, Debug)]
struct Variable {
    comment: Option<String>,
    type_t: String,
    name: String,
}

// The TypeReplacementMap is used to replace
// message types like GetStatusResponse witth status
// or builtin types like uint64 with uint64_t
#[derive(Clone, Debug, Default)]
struct TypeReplacementMap {
    map: HashMap<String, String>,
}

impl TypeReplacementMap {
    pub fn insert(&mut self, key: &str, val: &str) -> anyhow::Result<()> {
        if self.map.contains_key(val) {
            return Err(anyhow!(
                "Loop protection: Replacement map already contains value ({val}) for key ({key})"
            ));
        }
        if let Some(_) = self.map.insert(key.into(), val.into()) {
            Err(anyhow!("Duplicate key insertion for {key}"))
        } else {
            Ok(())
        }
    }
    pub fn map(&self, key: &str) -> String {
        self.map.get(key).cloned().unwrap_or(key.into()).into()
    }
    pub fn contains(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }
    pub fn from_proto(proto: &ComponentServiceProto, opts: &CliOpts) -> anyhow::Result<Self> {
        let mut repl_map = Self::default();
        // be careful about adding hardcoded replaces
        repl_map.insert("string", "std::string")?;
        repl_map.insert("uint64", "uint64_t")?;
        repl_map.insert("int64", "int64_t")?;
        repl_map.insert("int32", "int32_t")?;
        repl_map.insert("uint32", "uint32_t")?;
        repl_map.insert("uint8", "uint8_t")?;
        repl_map.insert("int8", "int8_t")?;
        repl_map.insert("common.v1.DoCommandResponse", "AttributeMap")?;
        for message in &proto.messages {
            let name = &message.name;
            // Only generate replacements for the responses
            if !name.ends_with("Response") {
                continue;
            }
            if message.fields.len() == 0 {
                repl_map.insert(name, VOID_T)?;
                continue;
            }
            let suggested_name = request_to_struct_name(&name);
            let new_name = if !opts.dont_prompt_for_struct_names {
                let entered_name = utils::string_prompt(&format!("Replacing struct name {name} with {suggested_name}. Press enter to confirm or suggest a new name:"));
                if entered_name.len() > 0 {
                    entered_name
                } else {
                    suggested_name
                }
            } else {
                suggested_name
            };
            repl_map.insert(name, &new_name)?;
        }
        for rpc in &proto.rpcs {
            let name = &rpc.name;
            let suggested_name = rpc_to_function_name(&name);
            let new_name = if !opts.dont_prompt_for_function_names {
                let entered_name = utils::string_prompt(&format!("Replacing function name {name} with {suggested_name}. Press enter to confirm or suggest a new name:"));
                if entered_name.len() > 0 {
                    entered_name
                } else {
                    suggested_name
                }
            } else {
                suggested_name
            };
            repl_map.insert(name, &new_name)?;
        }

        for enum_ in &proto.enums {
            let name = &enum_.name;
            let suggested_name = enum_to_enum_name(&name);
            let new_name = if !opts.dont_prompt_for_struct_names {
                let entered_name = utils::string_prompt(&format!("Replacing enum name {name} with {suggested_name}. Press enter to confirm or suggest a new name:"));
                if entered_name.len() > 0 {
                    entered_name
                } else {
                    suggested_name
                }
            } else {
                suggested_name
            };
            repl_map.insert(name, &new_name)?;
        }

        Ok(repl_map)
    }
}

impl TemplateInput {
    pub fn from_proto(proto: &ComponentServiceProto, opts: &CliOpts) -> anyhow::Result<Self> {
        let mut repl_map = TypeReplacementMap::from_proto(proto, opts)?;
        Ok(Self {
            component: ComponentDec::from_proto(proto, &mut repl_map, opts)
                .context("creating component dec")?,
        })
    }
}
impl ComponentDec {
    pub fn from_proto(
        proto: &ComponentServiceProto,
        repl_map: &mut TypeReplacementMap,
        opts: &CliOpts,
    ) -> anyhow::Result<Self> {
        let name = proto.name.clone();
        // ComponentService -> Component
        let name = name.trim_end_matches("Service").to_string();
        let mut member_fns = Vec::new();
        let mut from_proto_fns = Vec::new();
        let mut to_proto_fns = Vec::new();
        let mut structs = Vec::new();
        let mut rpcs = Vec::new();
        let mut enums = Vec::new();
        let message_fields_to_variables = |fields: &Vec<Field>| {
            fields
                .iter()
                .filter(|f| opts.include_name_field || f.name != NAME_PARAM)
                .filter(|f| opts.include_extra_field || f.name != EXTRA_PARAM)
                .map(|f| Variable {
                    comment: f.comment.clone(),
                    name: f.name.clone(),
                    type_t: repl_map.map(&f.type_t),
                })
                .collect()
        };

        // Use messages to create:
        //  - from_proto_fns
        //  - to_proto_fns
        //  - structs
        for message in &proto.messages {
            let orig_name = &message.name;

            // Only generate structs for the responses
            if !orig_name.ends_with("Response") {
                continue;
            }
            let name = repl_map.map(&orig_name);
            // dont gen if we are told that it is void
            if name.eq(VOID_T) {
                continue;
            }

            // If we have a remapping that isnt to void, we should gen from/to
            if repl_map.contains(&orig_name) {
                from_proto_fns.push(FnDec {
                    name: FROM_PROTO_T.into(),
                    return_t: name.clone(),
                    comment: None,
                    args: vec![Variable {
                        comment: None,
                        name: "proto".to_string(),
                        type_t: orig_name.to_string(),
                    }],
                });

                to_proto_fns.push(FnDec {
                    name: TO_PROTO_T.into(),
                    return_t: orig_name.into(),
                    comment: None,
                    args: vec![Variable {
                        comment: None,
                        name: name.to_owned(),
                        type_t: name.to_owned(),
                    }],
                });
            }
            let args = message_fields_to_variables(&message.fields);
            structs.push(StructDec {
                comment: message.comment.clone(),
                name: repl_map.map(&name),
                members: args,
            })
        }
        // Create enums
        for enum_ in &proto.enums {
            let orig_name = &enum_.name;
            let name = repl_map.map(&orig_name);
            enums.push(Enum {
                comment: enum_.comment.clone(),
                name,
                members: enum_.members.clone(),
            });
        }
        // Use rpcs to create rpcs to create:
        //  - rpcs
        //  - member_fns
        for rpc in &proto.rpcs {
            let orig_name = &rpc.name;
            let name = repl_map.map(&orig_name);
            // insert map for client / server recall
            rpcs.push(RpcDec {
                comment: rpc.comment.clone(),
                rpc_name: rpc.name.clone(),
                func_name: name.clone(),
                req_t: rpc.request_name.clone(),
                resp_t: rpc.response_name.clone(),
            });
            // find the corresponding request message,
            // get its members,
            // use those for the arguments to this fn
            let request_msg = &proto
                .messages
                .iter()
                .filter(|m| m.name == rpc.request_name)
                .next();
            let args = if let Some(request_msg) = request_msg {
                message_fields_to_variables(&request_msg.fields)
            } else {
                vec![Variable {
                    comment: Some("TODO".into()),
                    type_t: "ERROR".into(),
                    name: "TODO".into(),
                }]
            };
            member_fns.push(FnDec {
                comment: rpc.comment.clone(),
                name,
                return_t: repl_map.map(&rpc.response_name),
                args,
            })
        }

        Ok(Self {
            name,
            from_proto_fns,
            to_proto_fns,
            member_fns,
            structs,
            rpcs,
            enums,
        })
    }
}
// These could probably go in util
fn request_to_struct_name(request: &str) -> String {
    let mut struct_name = request.trim_end_matches("Response").to_lowercase();
    if struct_name.starts_with("get") {
        struct_name = struct_name[3..].to_string();
    } else if struct_name.starts_with("is") {
        struct_name = struct_name[2..].to_string();
        struct_name = format!("{struct_name}_status")
    } else {
        log::warn!("Warn: no rules to convert {request}")
    }
    struct_name
}

fn rpc_to_function_name(request: &str) -> String {
    request.to_case(convert_case::Case::Snake)
}

fn enum_to_enum_name(request: &str) -> String {
    request.to_case(convert_case::Case::Snake)
}

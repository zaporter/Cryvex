# Cryvex
The name `Cryvex` means nothing. It is a simple tool to generate extremely specific cpp files to wrap Viam Components for the viam-cpp-sdk from proto files. 

Unless someone told you about this project, you probably don't care about this. 


Limitations:
- multiline strings (like in get: "/.../") do not work. You must manually remove them.
- optional types are not optional
- it does not actually compile the proto so imports dont work
- formatting is bad at times (run clang-format)

Run on a proto file with `cargo run -- <PATH_TO_PROTO>`

See all cli args with `cargo run -- --help`

License: MIT

# Cryvex
The name `Cryvex` means nothing. It is a simple tool to generate extremely specific cpp files for the viam-cpp-sdk from proto files. 

If you found this project without someone telling you about it, you probably don't care about this project. 


Limitations:
- multiline strings (like in get: "/.../") do not work. You must manually remove them.
- optional types are not optional
- it does not actually compile the proto so imports dont work
- formatting is bad at times (run clang-format)

See all cli args with `cargo run -- --help`

License: MIT

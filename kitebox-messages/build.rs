fn main() {
    capnpc::CompilerCommand::new()
        .file("kitebox-messages.capnp")
        .run()
        .expect("compiling schema");
}

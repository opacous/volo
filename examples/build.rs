use std::env;
use std::error::Error;
use volo_cli::command::CliCommand;
use volo_cli::context::Context;


// in build.rs
macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

#[derive(Debug, Clone)]
enum BuildError {
    GeneralError(String)
}

fn main() -> Result<(), BuildError> {
    p!("TEST");
    // Constructed Path buffer
    let current_working_path =
        env::current_dir()
            .map_err(|err|
                BuildError::GeneralError(err.to_string())
            ).unwrap();
    let mut includes_path = current_working_path.clone();
    includes_path.push("proto");
    let mut individual_idl_path = current_working_path.clone();
    individual_idl_path.push("proto/hello.proto");

    let out_dir = std::env::var("OUT_DIR").expect("TODO: panic message");
    p!("{}", out_dir);

    // 0. Preparing my own Config Struct that ConfigBuilder needs
    let mut config_toml_path =current_working_path.clone();
    config_toml_path.push("volo.yml");

    // 1. Skips directly to build for a specific protobuf file!
    volo_build::ConfigBuilder::new(config_toml_path).write().expect("TODO: panic message");
    Ok(())
}

//     // Hardcoded just for hello.proto
//     let our_init = volo_cli::init::Init {
//         name: "hello_test".to_string(),
//         git: None,
//         r#ref: None,
//         includes: Some(vec![includes_path]),
//         idl: individual_idl_path,
//     };
//     p!("{:?}", our_init);
//
//
//     let our_context = Context { entry_name: "".to_string() };
//     let x = our_init.run(our_context);
//
//     p!("{:?}", x);
//
//     Ok(())
// }
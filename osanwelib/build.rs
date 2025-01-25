// use std::fs;
// use std::path::PathBuf;

fn main() {
//    let out_dir = PathBuf::from("src/generated");
//    fs::create_dir_all(&out_dir).unwrap();

//    println!("cargo:warning=Compiling Protobuf files...");

    prost_build::Config::new()
//        .out_dir(&out_dir)
        .compile_protos(
            &["proto/transaction_type_1.proto", "proto/transaction_type_2.proto"],
            &["proto/"],
        )
        .expect("Failed to compile Protobuf files");

//    println!("cargo:warning=Generated Protobuf files in: {:?}", out_dir);
}

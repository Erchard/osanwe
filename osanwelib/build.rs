fn main() {
    tonic_build::configure()
        .compile_protos(
            &["proto/transaction_pb.proto"], // Шлях до .proto файлу
            &["proto/"], // Директорія з .proto файлами
        )
        .expect("Failed to compile Protobuf files");
}

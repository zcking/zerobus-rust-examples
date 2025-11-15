use prost::Message;
use prost_types::DescriptorProto;

// Module for generated protobuf code
pub mod aws_raw_events {
    include!("../gen/rust/aws_raw_events.rs");
}

/// Load the protobuf descriptor from the embedded descriptor file
pub fn load_descriptor_proto(file_name: &str, message_name: &str) -> DescriptorProto {
    const DESCRIPTOR_BYTES: &[u8] = include_bytes!("../gen/descriptors/aws_raw_events.descriptor");

    let file_descriptor_set = prost_types::FileDescriptorSet::decode(DESCRIPTOR_BYTES)
        .expect("Failed to decode descriptor file");

    let file_descriptor_proto = file_descriptor_set
        .file
        .into_iter()
        .find(|f| f.name.as_ref().map(|n| n.as_str()) == Some(file_name))
        .expect("File descriptor not found");

    file_descriptor_proto
        .message_type
        .into_iter()
        .find(|m| m.name.as_ref().map(|n| n.as_str()) == Some(message_name))
        .expect("Message descriptor not found")
}


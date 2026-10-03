//! Tests of the `file_modules` parameter.

use super::*;

/// Builds a request for schemas that all declare the package `pkg`.
fn module_requests(file_modules: bool) -> ModuleRequestSet {
    let names = ["a/one.proto", "b/two.proto"];
    let protos: Vec<_> = names
        .iter()
        .map(|name| FileDescriptorProto {
            name: Some(name.to_string()),
            package: Some("pkg".to_string()),
            ..Default::default()
        })
        .collect();
    let raw_protos = RawProtos {
        proto_file: protos.iter().map(|proto| proto.encode_to_vec()).collect(),
    };
    ModuleRequestSet::new_decoded(
        names.iter().map(|name| name.to_string()),
        protos,
        raw_protos,
        "_",
        false,
        file_modules,
    )
}

#[test]
fn package_modules_merge_schemas_of_one_package() {
    let requests = module_requests(false);
    let paths: Vec<_> = requests
        .requests()
        .filter_map(|(_, request)| request.output_filepath())
        .collect();
    assert_eq!(paths, ["pkg/one.pb.rs"]);
}

#[test]
fn file_modules_write_each_schema_beside_its_source() {
    let requests = module_requests(true);
    let paths: Vec<_> = requests
        .requests()
        .filter_map(|(_, request)| request.output_filepath())
        .collect();
    assert_eq!(paths, ["a/one.pb.rs", "b/two.pb.rs"]);
    assert!(requests
        .requests()
        .all(|(_, request)| request.proto_package_name() == "pkg"));
}

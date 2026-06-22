use streamfy_controlplane_metadata::smartmodule::StreamfySemVersion;
use streamfy_connector_package::metadata::*;
use openapiv3::SchemaData;

#[test]
fn test_read_from_toml_file() {
    //given
    let path = format!("{}/tests/Connector.toml", env!("CARGO_MANIFEST_DIR"));

    //when
    let metadata = ConnectorMetadata::from_toml_file(path).unwrap();

    //then
    assert_eq!(
        metadata,
        ConnectorMetadata {
            direction: Direction::source(),
            deployment: Deployment::from_binary_name("json-test-connector"),
            package: ConnectorPackage {
                name: "json-test-connector".into(),
                group: "streamfy".into(),
                version: StreamfySemVersion::parse("0.1.0").unwrap(),
                streamfy: StreamfySemVersion::parse("0.10.0").unwrap(),
                api_version: StreamfySemVersion::parse("0.1.0").unwrap(),
                description: Some("Generate JSON generator".into()),
                license: Some("Apache-2.0".into()),
                visibility: ConnectorVisibility::Public,
            },
            custom_config: CustomConfigSchema::with(
                [(
                    "template",
                    openapiv3::Schema {
                        schema_data: SchemaData {
                            title: Some("template".to_owned()),
                            description: Some("JSON template".to_owned()),
                            ..Default::default()
                        },
                        schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(
                            Default::default()
                        ))
                    }
                )],
                ["template"]
            ),
        }
    )
}

#[test]
fn test_write_to_toml_file() {
    //given
    let file = tempfile::NamedTempFile::new().unwrap();
    let path = format!("{}/tests/Connector.toml", env!("CARGO_MANIFEST_DIR"));

    let metadata = ConnectorMetadata::from_toml_file(path).unwrap();
    //when
    metadata.to_toml_file(file.as_ref()).unwrap();

    let content = std::fs::read_to_string(file).unwrap();

    //then
    assert_eq!(
        content,
        r#"[package]
name = "json-test-connector"
group = "streamfy"
version = "0.1.0"
streamfy = "0.10.0"
apiVersion = "0.1.0"
description = "Generate JSON generator"
license = "Apache-2.0"
visibility = "public"

[direction]
source = true

[deployment]
binary = "json-test-connector"

[custom]
required = ["template"]

[custom.properties.template]
title = "template"
description = "JSON template"
type = "string"
"#
    );
}

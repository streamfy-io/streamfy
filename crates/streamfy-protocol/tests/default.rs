use streamfy_protocol::StreamfyDefault;

#[derive(StreamfyDefault, Debug)]
struct TestRecord {
    _value: i8,
    _value2: i8,
    #[streamfy(default = "4")]
    value3: i8,
    #[streamfy(default = "-1")]
    value4: i16,
}

#[test]
fn test_default() {
    let record = TestRecord::default();
    assert_eq!(record.value3, 4);
    assert_eq!(record.value4, -1);
}

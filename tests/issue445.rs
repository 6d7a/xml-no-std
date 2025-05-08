#[test]
fn issue_445() {
    use xml_no_std::Encoding;

    let parser = xml_no_std::ParserConfig::default().override_encoding(Some(Encoding::Utf16));

    let source: Vec<u8> = [0xEF, 0xBB, 0xBF, 0xFF, 0xFF].into();

    let mut rdr = parser.create_reader(source.iter());
    while let Ok(event) = rdr.next() {
        std::hint::black_box(event);
    }
}
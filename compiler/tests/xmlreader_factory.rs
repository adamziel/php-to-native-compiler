use php_compiler::run_source;

#[test]
fn xmlreader_from_stream_reads_memory_stream_and_comment_events() {
    let execution = run_source(
        r#"<?php
$h = fopen("php://memory", "w+");
fwrite($h, "<root><!--my comment--><child/></root>");
fseek($h, 0);

$reader = XMLReader::fromStream($h, encoding: "UTF-8");
while ($reader->read()) {
    if ($reader->nodeType === XMLReader::ELEMENT) {
        echo "Element: ", $reader->name, "\n";
    }
    if ($reader->nodeType === XMLReader::COMMENT) {
        echo "Comment: ", $reader->value, "\n";
    }
}
var_dump(ftell($h));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Element: root\nComment: my comment\nElement: child\nint(38)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn xmlreader_static_factories_validate_encoding_and_call_subclass_constructors() {
    let execution = run_source(
        r#"<?php
class CustomXMLReader extends XMLReader {
    public function __construct() {
        throw new Error("nope");
    }
}

foreach (["fromString", "fromStream"] as $method) {
    try {
        if ($method === "fromString") {
            CustomXMLReader::fromString("<root/>");
        } else {
            $h = fopen("php://memory", "w+");
            fwrite($h, "<root/>");
            fseek($h, 0);
            CustomXMLReader::fromStream($h, encoding: "UTF-8");
        }
    } catch (Throwable $e) {
        echo $method, ":", $e->getMessage(), "\n";
    }
}

$reader = new XMLReader();
try {
    $reader->XML("<root/>", "does not exist");
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "fromString:nope\n",
            "fromStream:nope\n",
            "XMLReader::XML(): Argument #2 ($encoding) must be a valid character encoding\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn xmlreader_schema_errors_and_static_open_overrides_match_reached_rows() {
    let execution = run_source(
        r#"<?php
$reader = new XMLReader();
try {
    $reader->setSchema("");
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
try {
    $reader->setSchema("schema-missing-file.xsd");
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
$reader->XML("<foo/>");
var_dump($reader->setSchema("schema-bad.xsd"));

class MyXMLReader extends XMLReader {
    public static function open(string $uri, ?string $encoding = null, int $flags = 0): bool|XMLReader {
        echo "overridden\n";
        return true;
    }
}
var_dump(MyXMLReader::open("asdf"));
$o = new MyXMLReader;
var_dump($o->open("asdf"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "XMLReader::setSchema(): Argument #1 ($filename) must not be empty\n",
            "Schema must be set prior to reading\n",
            "bool(false)\n",
            "overridden\n",
            "bool(true)\n",
            "overridden\n",
            "bool(true)\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn xmlreader_residual_error_and_reflection_surfaces_match_reached_rows() {
    let execution = run_source(
        r#"<?php
$reader = XMLReader::fromString('<root xmlns:ns1="urn:one"><book ns1:num="1"/></root>');
$reader->read();
$reader->read();
$reader->moveToNextAttribute();
try {
    $reader->getAttributeNs('num', null);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
try {
    $reader->moveToAttributeNs('num', null);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
try {
    clone $reader;
} catch (Throwable $e) {
    echo $e::class, ":", $e->getMessage(), "\n";
}
$rm = new ReflectionMethod(XMLReader::class, 'expand');
echo $rm->getNumberOfParameters(), "/", $rm->getNumberOfRequiredParameters(), "\n";
echo $reader->baseURI, "\n";
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains(
        "Deprecated: XMLReader::getAttributeNs(): Passing null to parameter #2 ($namespace) of type string is deprecated"
    ));
    assert!(execution
        .stdout
        .contains("XMLReader::getAttributeNs(): Argument #2 ($namespace) must not be empty\n"));
    assert!(execution.stdout.contains(
        "Deprecated: XMLReader::moveToAttributeNs(): Passing null to parameter #2 ($namespace) of type string is deprecated"
    ));
    assert!(execution
        .stdout
        .contains("XMLReader::moveToAttributeNs(): Argument #2 ($namespace) must not be empty\n"));
    assert!(execution
        .stdout
        .contains("Error:Trying to clone an uncloneable object of class XMLReader\n"));
    assert!(execution.stdout.contains("1/0\nstring://\n"));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

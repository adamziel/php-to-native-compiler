use php_compiler::run_source;

#[test]
fn dom_attr_constructor_and_properties_match_direct_phpt_surface() {
    let execution = run_source(
        r#"<?php
try {
    new DOMAttr();
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
$attr = new DOMAttr('category', 'books');
echo $attr->name, "\n";
echo $attr->value, "\n";
$empty = new DOMAttr('empty');
echo $empty->value, "\n";
$empty->value = 1;
echo $empty->value, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "DOMAttr::__construct() expects at least 1 argument, 0 given\n",
            "category\n",
            "books\n",
            "\n",
            "1\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dom_document_attribute_creation_and_serialization_are_supported() {
    let execution = run_source(
        r#"<?php
$doc = new DOMDocument;
$node = $doc->createElement("para");
$doc->appendChild($node);
$attr = $doc->createAttribute("hahaha");
$node->appendChild($attr);
echo $doc->saveXML();

$created = $doc->createAttribute('string');
echo get_class($created), "\n";

try {
    $doc->createAttribute(0);
} catch (DOMException $e) {
    echo $e->getCode(), ":", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "<?xml version=\"1.0\"?>\n",
            "<para hahaha=\"\"/>\n",
            "DOMAttr\n",
            "5:Invalid Character Error\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dom_element_attributes_track_names_and_owner_lifetime() {
    let execution = run_source(
        r#"<?php
$document = new DOMDocument;
$root = $document->createElement('root');
$document->appendChild($root);
$attr = $root->setAttribute('category', 'books');
var_dump($root->hasAttribute('category'));
echo $root->getAttribute('category'), "\n";
print_r($root->getAttributeNames());
var_dump($root->hasAttributes());
$document->removeChild($root);
var_dump($attr->ownerElement);

$element = new DOMElement("container");
var_dump($element->toggleAttribute('foo', true));
$dom = new DOMDocument;
$element = $dom->importNode($element, true);
echo $dom->saveXML($element), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(true)\n",
            "books\n",
            "Array\n",
            "(\n",
            "    [0] => category\n",
            ")\n",
            "bool(true)\n",
            "NULL\n",
            "bool(true)\n",
            "<container foo=\"\"/>\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dom_document_type_invalid_state_properties_raise_dom_exception() {
    let execution = run_source(
        r#"<?php
function marker($name) {
    echo "arg:$name\n";
    return $name;
}

$direct = new DOMDocumentType(marker("direct"));
try {
    $direct->name;
} catch (DOMException $e) {
    echo "name|", $e->getCode(), "|", $e->getMessage(), "\n";
}

foreach (array("entities", "notations", "publicId", "systemId", "internalSubset") as $property) {
    $doctype = new DOMDocumentType();
    try {
        $doctype->$property;
    } catch (DOMException $e) {
        echo $property, "|", $e->getCode(), "|", $e->getMessage(), "\n";
    }
}

echo class_exists("DOMDocumentType") ? "class\n" : "missing\n";
$extension = new ReflectionExtension("dom");
echo in_array("DOMDocumentType", $extension->getClassNames(), true) ? "listed\n" : "not-listed\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "arg:direct\n",
            "name|11|Invalid State Error\n",
            "entities|11|Invalid State Error\n",
            "notations|11|Invalid State Error\n",
            "publicId|11|Invalid State Error\n",
            "systemId|11|Invalid State Error\n",
            "internalSubset|11|Invalid State Error\n",
            "class\n",
            "listed\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

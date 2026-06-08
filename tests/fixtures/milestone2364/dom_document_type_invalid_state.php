<?php
function show_doctype_read($property) {
    $doctype = new DOMDocumentType();
    try {
        echo $doctype->$property, "\n";
    } catch (DOMException $e) {
        echo $property, "|", $e->getCode(), "|", $e->getMessage(), "\n";
    }
}

function side_effect($label) {
    echo "arg:$label\n";
    return $label;
}

$doctype = new DOMDocumentType(side_effect("ignored"));
try {
    $doctype->name;
} catch (DOMException $e) {
    echo "name|", $e->getCode(), "|", $e->getMessage(), "\n";
}

show_doctype_read("entities");
show_doctype_read("notations");
show_doctype_read("publicId");
show_doctype_read("systemId");
show_doctype_read("internalSubset");

echo class_exists("DOMDocumentType") ? "class\n" : "missing\n";
$dom = new ReflectionExtension("dom");
echo in_array("DOMDocumentType", $dom->getClassNames(), true) ? "listed" : "missing";

<?php
function helper_before() {}

/**
 * WordPress-style callback metadata.
 */
function reflected_callback($hook) {
    return $hook;
}

function no_doc_comment() {}

function yn($value) {
    return $value ? "1" : "0";
}

function doc_line($label, $function) {
    $doc = $function->getDocComment();
    echo $label, "|", yn($doc !== false), "|", str_replace("\n", "\\n", $doc), "\n";
}

$function = new ReflectionFunction("reflected_callback");
$suffix = "tests/fixtures/milestone1491/function_reflection_source_metadata.php";
echo "source|", substr($function->getFileName(), -strlen($suffix)), "|", $function->getStartLine(), "|", $function->getEndLine(), "\n";
doc_line("doc", $function);
$plain = new ReflectionFunction("no_doc_comment");
echo "plain|", $plain->getStartLine(), "|", $plain->getEndLine(), "|", yn($plain->getDocComment() === false);

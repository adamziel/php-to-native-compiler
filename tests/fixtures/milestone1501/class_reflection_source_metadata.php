<?php
/**
 * Base class metadata.
 */
class PluginBase {
}

/**
 * WordPress hook plugin metadata.
 */
class HookPlugin extends PluginBase {
    public function boot() {}
}

/**
 * Hook contract metadata.
 */
interface HookContract {
    public function boot();
}

/**
 * Hook trait metadata.
 */
trait HookTools {
    public function helper() {}
}

class PlainClass {}

function yn($value) {
    return $value ? "1" : "0";
}

function class_doc_line($label, $class) {
    $doc = $class->getDocComment();
    echo $label, "|", yn($doc !== false), "|", str_replace("\n", "\\n", $doc), "\n";
}

$suffix = "tests/fixtures/milestone1501/class_reflection_source_metadata.php";
$class = new ReflectionClass(HookPlugin::class);
echo "class-source|", substr($class->getFileName(), -strlen($suffix)), "|", $class->getStartLine(), "|", $class->getEndLine(), "\n";
class_doc_line("class-doc", $class);
$parent = $class->getParentClass();
echo "parent-lines|", $parent->getStartLine(), "|", $parent->getEndLine(), "\n";
$interface = new ReflectionClass(HookContract::class);
echo "interface-lines|", $interface->getStartLine(), "|", $interface->getEndLine(), "\n";
class_doc_line("interface-doc", $interface);
$trait = new ReflectionClass(HookTools::class);
echo "trait-lines|", $trait->getStartLine(), "|", $trait->getEndLine(), "\n";
class_doc_line("trait-doc", $trait);
$plain = new ReflectionClass(PlainClass::class);
echo "plain|", $plain->getStartLine(), "|", $plain->getEndLine(), "|", yn($plain->getDocComment() === false);

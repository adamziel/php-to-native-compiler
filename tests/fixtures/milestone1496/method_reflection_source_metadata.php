<?php
class HookBase {
    /**
     * Parent hook metadata.
     */
    public function inherited($value) {
        return $value;
    }
}

class HookPlugin extends HookBase {
    /**
     * Registers WordPress hooks.
     */
    public function register($hook) {
        return $hook;
    }

    public function noDoc() {}
}

function yn($value) {
    return $value ? "1" : "0";
}

function doc_line($label, $method) {
    $doc = $method->getDocComment();
    echo $label, "|", yn($doc !== false), "|", str_replace("\n", "\\n", $doc), "\n";
}

$method = new ReflectionMethod(HookPlugin::class, "register");
$suffix = "tests/fixtures/milestone1496/method_reflection_source_metadata.php";
echo "source|", substr($method->getFileName(), -strlen($suffix)), "|", $method->getStartLine(), "|", $method->getEndLine(), "\n";
doc_line("doc", $method);
$inherited = new ReflectionMethod(HookPlugin::class, "inherited");
doc_line("inherited", $inherited);
echo "inherited-lines|", $inherited->getStartLine(), "|", $inherited->getEndLine(), "\n";
$plain = new ReflectionMethod(HookPlugin::class, "noDoc");
echo "plain|", $plain->getStartLine(), "|", $plain->getEndLine(), "|", yn($plain->getDocComment() === false);

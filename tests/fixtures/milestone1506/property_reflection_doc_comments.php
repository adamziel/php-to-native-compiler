<?php
class Base {
    /**
     * Shared cache metadata.
     */
    protected static $cache = "warm";
}

class Plugin extends Base {
    /**
     * Public hook name metadata.
     */
    public $name = "hook";

    public $plain = "none";
}

function yn($value) {
    return $value ? "1" : "0";
}

function doc_line($label, $property) {
    $doc = $property->getDocComment();
    echo $label, "|", $property->getName(), "|", $property->getDeclaringClass()->getName(), "|", yn($doc !== false), "|", str_replace("\n", "\\n", $doc), "\n";
}

doc_line("direct", new ReflectionProperty(Plugin::class, "name"));
doc_line("inherited", new ReflectionProperty(Plugin::class, "cache"));
$plain = new ReflectionProperty(Plugin::class, "plain");
echo "plain|", $plain->getName(), "|", yn($plain->getDocComment() === false);

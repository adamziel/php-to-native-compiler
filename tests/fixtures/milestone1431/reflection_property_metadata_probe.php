<?php
class Base {
    public $id = "base";
    protected static $cache = "warm";
    private $secret = "hidden";
}

class Plugin extends Base {
    public $name = "hook";
    protected $items = array("a" => 1);
    private static $flag = true;
}

function yn($value) {
    return $value ? "1" : "0";
}

function default_label($value) {
    if (is_array($value)) {
        return "array:" . count($value);
    }
    if (is_bool($value)) {
        return yn($value);
    }
    if ($value === null) {
        return "null";
    }
    return $value;
}

function line($label, $property, $ending = "\n") {
    echo $label, "|", get_class($property), "|", $property->getName(), "|", $property->getDeclaringClass()->getName(), "|", $property->getModifiers(), "|", yn($property->isPublic()), yn($property->isProtected()), yn($property->isPrivate()), yn($property->isStatic()), "|", yn($property->hasDefaultValue()), "|", default_label($property->getDefaultValue()), "|", yn($property->hasType()), yn($property->getType() === null), $ending;
}

$rc = new ReflectionClass(Plugin::class);
echo "constants|", ReflectionProperty::IS_PUBLIC, "|", ReflectionProperty::IS_PROTECTED, "|", ReflectionProperty::IS_PRIVATE, "|", ReflectionProperty::IS_STATIC, "\n";
echo "has|", yn($rc->hasProperty("items")), yn($rc->hasProperty("secret")), "\n";
line("direct", new ReflectionProperty(Plugin::class, "name"));
line("object", new ReflectionProperty(new Plugin(), "cache"));
line("get", $rc->getProperty("flag"));
$properties = $rc->getProperties();
$count = count($properties);
foreach ($properties as $index => $property) {
    line("list", $property, $index + 1 === $count ? "" : "\n");
}

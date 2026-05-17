<?php
class Base {
    public string $id = "base";
    protected static ?string $cache = null;
}

class Plugin extends Base {
    public ?string $name = null;
    protected array $items = array("a" => 1);
    private static bool $flag = true;
    public ?Plugin $peer = null;
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

function type_label($property) {
    $type = $property->getType();
    if ($type === null) {
        return "none";
    }
    return get_class($type) . ":" . $type->getName() . ":" . yn($type->allowsNull()) . yn($type->isBuiltin()) . yn($type instanceof ReflectionType);
}

function line($label, $property, $ending = "\n") {
    echo $label, "|", $property->getName(), "|", yn($property->hasType()), "|", type_label($property), "|", yn($property->hasDefaultValue()), "|", default_label($property->getDefaultValue()), $ending;
}

$rc = new ReflectionClass(Plugin::class);
line("direct", new ReflectionProperty(Plugin::class, "name"));
line("object", new ReflectionProperty(new Plugin(), "cache"));
line("get", $rc->getProperty("flag"));
$properties = $rc->getProperties();
$count = count($properties);
foreach ($properties as $index => $property) {
    line("list", $property, $index + 1 === $count ? "" : "\n");
}

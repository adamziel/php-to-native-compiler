<?php
interface HookContract {}
interface TaggedContract {}
class Hook implements HookContract {}
class TaggedHook extends Hook implements TaggedContract {}
class OtherHook {}

class Registry {
    public HookContract|OtherHook|null $union = null;
    public HookContract&TaggedContract $intersection;
}

function yn($value) {
    return $value ? "1" : "0";
}

function type_names($type) {
    $names = array();
    foreach ($type->getTypes() as $inner) {
        $names[] = $inner->getName() . ":" . yn($inner->isBuiltin()) . ":" . yn($inner->allowsNull());
    }
    return implode(",", $names);
}

$registry = new Registry();
$registry->union = new OtherHook();
$registry->intersection = new TaggedHook();

$union = (new ReflectionProperty(Registry::class, "union"))->getType();
$intersection = (new ReflectionProperty(Registry::class, "intersection"))->getType();

echo get_class($registry->union), "|", get_class($registry->intersection), "\n";
echo get_class($union), "|", yn($union instanceof ReflectionType), "|", yn($union->allowsNull()), "|", type_names($union), "\n";
echo get_class($intersection), "|", yn($intersection instanceof ReflectionType), "|", yn($intersection->allowsNull()), "|", type_names($intersection);

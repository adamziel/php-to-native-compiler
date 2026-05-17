<?php
trait RegistersHooks {}
trait AddsFilters {}

class Plugin {
    use RegistersHooks, AddsFilters;
}

interface Hookable {}

function yn($value) {
    return $value ? "1" : "0";
}

$class = new ReflectionClass(Plugin::class);
foreach ($class->getTraitNames() as $index => $name) {
    echo "name|", $index, "|", $name, "\n";
}
foreach ($class->getTraits() as $key => $trait) {
    echo "trait|", $key, "|", get_class($trait), "|", $trait->getName(), "|", yn($trait->isTrait()), "|", $trait->getShortName(), "\n";
}

$interface = new ReflectionClass(Hookable::class);
echo "interface|", count($interface->getTraitNames()), "|", count($interface->getTraits()), "\n";

$trait = new ReflectionClass(RegistersHooks::class);
echo "trait-empty|", count($trait->getTraitNames()), "|", count($trait->getTraits());

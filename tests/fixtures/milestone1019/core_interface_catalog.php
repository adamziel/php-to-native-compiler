<?php
foreach (array("Traversable", "IteratorAggregate", "Iterator", "Serializable", "ArrayAccess", "Countable", "Stringable") as $name) {
    echo interface_exists($name) ? $name . ":yes\n" : $name . ":no\n";
}

echo interface_exists("DefinitelyMissingInterface") ? "missing:yes" : "missing:no";

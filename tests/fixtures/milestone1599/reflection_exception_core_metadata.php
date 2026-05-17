<?php
echo class_exists("ReflectionException") ? "exists\n" : "missing\n";
echo is_subclass_of("ReflectionException", "Exception") ? "extends\n" : "no-parent\n";
echo get_parent_class("ReflectionException"), "\n";
$reflection = new ReflectionClass("ReflectionException");
echo $reflection->getName(), "|", $reflection->getParentClass()->getName(), "|", ($reflection->isInstantiable() ? "1" : "0"), "\n";
echo in_array("ReflectionException", get_declared_classes(), true) ? "declared" : "not-declared";

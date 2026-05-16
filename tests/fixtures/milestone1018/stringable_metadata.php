<?php
class Label {
    public function __toString() {
        return "label";
    }
}

class ChildLabel extends Label {}

class ExplicitLabel implements Stringable {
    public function __toString() {
        return "explicit";
    }
}

class Plain {}

$label = new Label();
$child = new ChildLabel();
$explicit = new ExplicitLabel();
$plain = new Plain();

echo interface_exists("Stringable") ? "interface\n" : "missing\n";
echo $label instanceof Stringable ? "instanceof\n" : "no-instanceof\n";
echo is_a($child, "Stringable") ? "child:is-a\n" : "child:no\n";
echo is_subclass_of("ChildLabel", "Stringable") ? "child:subclass\n" : "child:no-subclass\n";
echo is_a($explicit, "Stringable") ? "explicit:is-a\n" : "explicit:no\n";
echo is_a($plain, "Stringable") ? "plain:is-a\n" : "plain:no\n";
echo in_array("Stringable", get_declared_interfaces(), true) ? "declared\n" : "not-declared\n";

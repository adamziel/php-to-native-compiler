<?php
function yn($value) {
    return $value ? "1" : "0";
}

class BaseProto {
    public function label() {}
    private function hidden() {}
}

interface ProtoContract {
    public function run();
}

class ChildProto extends BaseProto implements ProtoContract {
    public function label() {}
    public function run() {}
    private function hidden() {}
}

$class = new ReflectionClass(ChildProto::class);

function show_method($label, $method) {
    echo $label, "|", $method->class, "::", $method->name, "|", $method->getName(), "|", $method->getDeclaringClass()->getName(), "\n";
}

$label = $class->getMethod("label");
show_method("label", $label);
echo "interpolated|$label->class::$label->name()\n";
$labelPrototype = $label->getPrototype();
show_method("label-prototype", $labelPrototype);

$run = $class->getMethod("run");
show_method("run", $run);
echo "run-has-prototype|", yn($run->hasPrototype()), "\n";
$runPrototype = $run->getPrototype();
show_method("run-prototype", $runPrototype);

$hidden = $class->getMethod("hidden");
echo "hidden-has-prototype|", yn($hidden->hasPrototype()), "\n";
try {
    $hidden->getPrototype();
} catch (ReflectionException $exception) {
    echo "hidden-prototype|", $exception->getMessage(), "\n";
}
try {
    $class->getMethod("missing");
} catch (ReflectionException $exception) {
    echo "missing|", $exception->getMessage(), "\n";
}

$sort = new ReflectionFunction("sort");
echo "extension|", get_class($sort->getExtension()), "|", $sort->getExtension()->getName(), "\n";
function local_proto_fn() {}
$extension = (new ReflectionFunction("local_proto_fn"))->getExtension();
echo $extension === null ? "NULL" : get_class($extension);

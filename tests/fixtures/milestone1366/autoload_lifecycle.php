<?php
function FirstLoader($name) {
    echo "first:", $name, "\n";
}

function OtherLoader($name) {
    echo "other:", $name, "\n";
}

class StaticLoader {
    public static function load($name) {
        echo "static:", $name, "\n";
    }
}

class ObjectLoader {
    public function load($name) {
        echo "object:", $name, "\n";
    }

    public function __invoke($name) {
        echo "invoke:", $name, "\n";
    }
}

$loader = new ObjectLoader();
echo count(spl_autoload_functions()), "\n";
spl_autoload_register("FirstLoader");
spl_autoload_register(array("StaticLoader", "load"));
spl_autoload_register(array($loader, "load"));
spl_autoload_register($loader, true, true);

$callbacks = spl_autoload_functions();
echo count($callbacks), "\n";
echo is_object($callbacks[0]) ? get_class($callbacks[0]) : "not-object", "\n";
echo $callbacks[1], "\n";
echo $callbacks[2][0], "::", $callbacks[2][1], "\n";
echo get_class($callbacks[3][0]), "::", $callbacks[3][1], "\n";

echo class_exists("MissingOne") ? "loaded\n" : "missing\n";
echo spl_autoload_unregister($loader) ? "removed-invoke\n" : "missing-invoke\n";
echo spl_autoload_unregister(array("StaticLoader", "load")) ? "removed-static\n" : "missing-static\n";
echo spl_autoload_unregister("OtherLoader") ? "removed-missing\n" : "missing-callback\n";

$callbacks = spl_autoload_functions();
echo count($callbacks), "\n";
echo class_exists("MissingTwo") ? "loaded" : "missing";

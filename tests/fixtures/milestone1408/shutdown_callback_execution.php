<?php
function first_shutdown($value) {
    echo "shutdown:", $value, "\n";
    register_shutdown_function("late_shutdown", "late");
}

function late_shutdown($value) {
    echo "shutdown:", $value, "\n";
}

class ShutdownHandler {
    public $prefix;

    public function __construct($prefix) {
        $this->prefix = $prefix;
    }

    public function handle($value) {
        echo $this->prefix, ":", $value, "\n";
    }

    public function __destruct() {
        echo "destruct";
    }
}

$handler = new ShutdownHandler("method");
register_shutdown_function("first_shutdown", "first");
register_shutdown_function([$handler, "handle"], "second");
echo "body\n";

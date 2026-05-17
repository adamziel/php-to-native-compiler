<?php
class BaseLogger {
    public function label(string $value) {
        return "base:" . $value;
    }
}

class ChildLogger extends BaseLogger {
    public function label($value) {
        return "child:" . $value;
    }
}

class BaseId {
    public function id() {
        return "base-id";
    }
}

class ChildId extends BaseId {
    public function id(): string {
        return "child-id";
    }
}

$logger = new ChildLogger();
$id = new ChildId();
echo $logger->label("hook"), "|";
echo method_exists($id, "id") ? "id-method" : "missing";
echo "|";
echo method_exists($logger, "label") ? "label-method" : "missing";

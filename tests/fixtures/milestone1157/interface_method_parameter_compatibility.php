<?php
interface Logger {
    public function log($message);
}

class Service implements Logger {
    public function log($message, $context = "default") {
        return "log:" . $message . ":" . $context;
    }
}

$service = new Service();
echo $service->log("ok"), "\n";
echo $service->log("ok", "custom");

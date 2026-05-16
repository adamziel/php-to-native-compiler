<?php
interface Logger {
    public function log($message);
}

class Service implements Logger {
    public function log($message) {
        return "log:" . $message;
    }
}

$service = new Service();
echo $service->log("ok");

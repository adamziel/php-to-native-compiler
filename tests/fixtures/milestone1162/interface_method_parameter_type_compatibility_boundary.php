<?php
interface Logger {
    public function log($message);
}

class Service implements Logger {
    public function log(string $message) {}
}

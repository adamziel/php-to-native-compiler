<?php
interface Logger {
    public function log(string $message);
}

class ExactLogger implements Logger {
    public function log(string $message) {}
}

class BroadLogger implements Logger {
    public function log($message) {}
}

echo "registered";

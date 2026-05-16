<?php
interface Logger {
    public function log($message);
}

class Service implements Logger {
    public static function log($message) {}
}

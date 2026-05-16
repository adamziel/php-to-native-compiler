<?php
interface Logger {
    public function log($message);
}

class Service implements Logger {}

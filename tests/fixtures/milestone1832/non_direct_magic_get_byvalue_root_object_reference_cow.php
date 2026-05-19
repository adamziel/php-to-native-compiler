<?php
error_reporting(0);

class Milestone1832Payload {
    public $value = "seed";
}

class Milestone1832Box {
    public $payload;
    public $hits = array();

    public function __construct() {
        $this->payload = new Milestone1832Payload();
    }

    public function __get($name) {
        $this->hits[] = $name;
        return $this->payload;
    }
}

$box = new Milestone1832Box();
$holders = array("box" => $box);
$alias =& $holders["box"]->missing;
$alias->value = "changed";
$read = $box->missing->value;

echo $box->payload->value, "|", $read, "|hits=", count($box->hits);

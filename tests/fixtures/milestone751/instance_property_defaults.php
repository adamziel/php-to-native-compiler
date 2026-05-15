<?php
class Defaults {
    public $_nplurals = 2;
    public $name = "Ada";
    protected $secret = "ok";
    private $token = "sealed";

    public function read() {
        return $this->_nplurals . ":" . $this->name . ":" . $this->secret . ":" . $this->token;
    }
}

$first = new Defaults();
$second = new Defaults();
$first->_nplurals = 3;
echo $first->read(), "\n";
echo $second->read(), "\n";
print_r(get_class_vars("Defaults"));
print_r(get_object_vars($second));
echo "done";

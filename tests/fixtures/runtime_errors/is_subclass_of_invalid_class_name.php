<?php
class Box {}
$box = new Box();
var_dump(is_subclass_of($box, 42));

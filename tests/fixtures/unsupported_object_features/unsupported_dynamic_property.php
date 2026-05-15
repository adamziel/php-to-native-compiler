<?php
class Box {}
$box = new Box();
$property = "missing";
$box->$property = 1;

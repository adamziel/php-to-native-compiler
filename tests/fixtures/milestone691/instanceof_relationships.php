<?php
class Box {}
class Child extends Box {}
class Other {}

$box = new Box();
$child = new Child();

echo $box instanceof Box ? "1" : "0";
echo $child instanceof Child ? "1" : "0";
echo $child instanceof Box ? "1" : "0";
echo $box instanceof Child ? "1" : "0";
echo $child instanceof Other ? "1" : "0";
echo $child instanceof Missing ? "1" : "0";
echo "x" instanceof Box ? "1" : "0";
echo $child INSTANCEOF box ? "1" : "0";

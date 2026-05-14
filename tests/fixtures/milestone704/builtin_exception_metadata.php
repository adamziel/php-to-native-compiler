<?php
echo class_exists("Exception"), "\n";
$exception = new Exception();
echo get_class($exception), "\n";

class FixtureException extends Exception {}
$fixture = new FixtureException();
echo get_parent_class($fixture), "\n";
echo is_a($fixture, "Exception") ? "yes" : "no", "\n";

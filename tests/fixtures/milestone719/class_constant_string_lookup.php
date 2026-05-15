<?php
class App_Config {
    const VERSION = 7;
    public const LABEL = "ok";
}
class Child_Config extends App_Config {}

$name = "VERSION";
echo defined("App_Config::$name") ? "1" : "0";
echo "|", constant("App_Config::$name"), "\n";
echo defined("\\App_Config::LABEL") ? "1" : "0";
echo "|", constant("\\App_Config::LABEL"), "\n";
echo defined("Child_Config::VERSION") ? "1" : "0";
echo "|", constant("Child_Config::VERSION"), "\n";
echo defined("App_Config::MISSING") ? "1" : "0";
echo "|", defined("Missing_Config::VERSION") ? "1" : "0", "\n";

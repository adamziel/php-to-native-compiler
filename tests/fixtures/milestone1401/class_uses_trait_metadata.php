<?php
namespace App;

trait RegistersHooks {}
trait AddsFilters {}
trait ParentOnly {}

class BasePlugin {
    use ParentOnly;
}

class Plugin extends BasePlugin {
    use RegistersHooks, AddsFilters;
}

$plugin = new Plugin();
print_r(class_uses($plugin));
print_r(class_uses("App\\Plugin", false));

$call = "class_uses";
$dynamic = $call("App\\Plugin");
echo count($dynamic), "\n";
echo isset($dynamic["App\\RegistersHooks"]) ? "registers\n" : "missing\n";
echo isset($dynamic["App\\ParentOnly"]) ? "parent-present" : "parent-not-listed";

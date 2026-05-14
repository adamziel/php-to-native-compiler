<?php
namespace Synthetic\WordPress;

class BaseLoader {}
class Loader extends BaseLoader {}

$loader = new Loader();
echo Loader::class, "\n";
echo get_parent_class($loader), "\n";
echo is_subclass_of($loader, BaseLoader::class) ? "yes" : "no";

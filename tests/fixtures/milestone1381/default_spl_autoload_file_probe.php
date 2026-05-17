<?php
spl_autoload_extensions(".class.inc,.inc");
spl_autoload("Wp_Loader");
$direct = new Wp_Loader();
echo $direct->name, "\n";

$call = "spl_autoload";
$call("Acme\\Plugin", null);
echo class_exists("Acme\\Plugin", false) ? "namespace\n" : "missing-namespace\n";

spl_autoload_extensions(".autoload.inc");
spl_autoload_register("spl_autoload");
$registered = new RegisteredBox();
echo $registered->name;

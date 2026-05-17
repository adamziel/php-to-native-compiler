<?php
spl_autoload_extensions(".class.inc");
spl_autoload_register("spl_autoload");

echo class_exists("Wp_Alias", false) ? "pre-alias\n" : "pre-missing\n";
echo class_alias("Wp_Original", "Wp_Alias") ? "alias-ok\n" : "alias-fail\n";
echo class_exists("Wp_Alias", false) ? "alias-exists\n" : "alias-missing\n";
$box = new Wp_Alias();
echo get_class($box), ":", $box->name, "\n";
echo is_a($box, "Wp_Alias") ? "is-a-alias" : "not-alias";

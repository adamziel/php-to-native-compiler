<?php
echo dirname("/tmp/wordpress/wp-includes/sodium_compat/autoload.php"), "\n";
echo dirname("/tmp/wordpress/wp-includes/sodium_compat/"), "\n";
echo "[", dirname("autoload.php"), "]\n";
echo dirname("/a/b/c.php", 2), "\n";

$call = "dirname";
echo $call("/a/b//c.php");

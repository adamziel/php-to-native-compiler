<?php
echo basename("/tmp/wordpress/wp-includes/plugin.php"), "\n";
echo basename("/tmp/wordpress/wp-includes/"), "\n";
echo "[", basename("autoload.php"), "]\n";
echo "[", basename(""), "]\n";
echo "[", basename("/"), "]\n";
echo basename("/a/b/c.php", ".php"), "\n";

$call = "basename";
echo $call("/a/b//c.php");

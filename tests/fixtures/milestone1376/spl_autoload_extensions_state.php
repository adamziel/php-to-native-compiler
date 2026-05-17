<?php
echo spl_autoload_extensions(), "\n";
echo spl_autoload_extensions(".php,.inc"), "\n";
echo spl_autoload_extensions(), "\n";
echo spl_autoload_extensions(null), "\n";
$call = "spl_autoload_extensions";
echo $call(".class.php"), "\n";
echo spl_autoload_extensions();

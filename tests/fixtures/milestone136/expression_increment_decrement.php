<?php
$int = 10;
echo ++$int, ":", $int, "\n";
echo $int++, ":", $int, "\n";
echo --$int, ":", $int, "\n";
echo $int--, ":", $int, "\n";

$value = 2;
echo $value++ + 10, ":", $value, "\n";
echo ++$value + 10, ":", $value, "\n";
echo $value++ + $value++, ":", $value, "\n";

$side = 1;
++$side + 10;
echo "side:", $side, "\n";
$side++ + 10;
echo "side:", $side, "\n";

$float = 1.5;
echo $float++, ":", $float, "\n";
echo --$float, ":", $float, "\n";

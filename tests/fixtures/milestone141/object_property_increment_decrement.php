<?php
class Box {
    public $value;
    public $float;
    public $i;
    public $sum;
}

$box = new Box();
$box->value = 10;
++$box->value;
echo $box->value, "\n";
echo $box->value++, ":", $box->value, "\n";
echo --$box->value, ":", $box->value, "\n";
echo $box->value--, ":", $box->value, "\n";

$box->float = 1.5;
echo $box->float++, ":", $box->float, "\n";
echo --$box->float, ":", $box->float, "\n";

$box->value = 1;
++$box->value + 10;
echo "side:", $box->value, "\n";
$box->value++ + 10;
echo "side:", $box->value, "\n";

$box->sum = 0;
for ($box->i = 0; $box->i < 3; $box->i++) {
    $box->sum += $box->i;
}
echo $box->sum, ":", $box->i;

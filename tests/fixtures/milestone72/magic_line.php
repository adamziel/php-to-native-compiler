<?php
echo __LINE__, "\n";
$line = __LINE__;
echo $line, "\n";

function default_line($line = __LINE__) {
    echo $line, "\n";
    echo __LINE__, "\n";
}

const DECLARED_LINE = __LINE__;

default_line();
echo DECLARED_LINE, "\n";

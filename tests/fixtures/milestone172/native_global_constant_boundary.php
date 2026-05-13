<?php
echo ARRAY_FILTER_USE_KEY, "\n";
define("RUNTIME_BASE", 3);
const FROM_DEFINE = RUNTIME_BASE + 1;
const NAME = "compiler", MODE = ARRAY_FILTER_USE_BOTH;
echo FROM_DEFINE, "|", NAME, "|", MODE, "\n";
echo constant("RUNTIME_BASE"), "|", defined("RUNTIME_BASE"), "|", defined("MISSING_CONST");

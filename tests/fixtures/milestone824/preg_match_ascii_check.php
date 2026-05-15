<?php
$pattern = '/[^\x00-\x7F]/';

echo preg_match($pattern, "SELECT option_name"), "|";
echo preg_match($pattern, "café");

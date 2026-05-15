<?php
header("Last-Modified: today");
$result = header_remove("Last-Modified");
echo $result === null ? "null" : "not-null";

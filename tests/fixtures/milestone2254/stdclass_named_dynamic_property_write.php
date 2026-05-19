<?php
$data = new stdClass();
$data->answer = 42;
echo $data->answer, "|";

$data->args = array("x");
echo $data->args[0];

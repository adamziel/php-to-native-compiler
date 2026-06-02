<?php
var_dump(INF);
var_dump(json_encode(INF));
var_dump(json_last_error(), json_last_error_msg());
var_dump(json_encode(INF, JSON_PARTIAL_OUTPUT_ON_ERROR));
var_dump(json_last_error(), json_last_error_msg());
var_dump(json_encode(-INF, JSON_PARTIAL_OUTPUT_ON_ERROR));
var_dump(json_encode(NAN, JSON_PARTIAL_OUTPUT_ON_ERROR));
var_dump(json_encode(array(INF, -INF, NAN), JSON_PARTIAL_OUTPUT_ON_ERROR));
$obj = new stdClass;
$obj->x = INF;
var_dump(json_encode($obj, JSON_PARTIAL_OUTPUT_ON_ERROR));
var_dump(json_last_error(), json_last_error_msg());
echo json_encode(array("x" => INF), JSON_PARTIAL_OUTPUT_ON_ERROR | JSON_PRETTY_PRINT);

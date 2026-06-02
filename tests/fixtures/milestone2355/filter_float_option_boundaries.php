<?php
function filter_cb($var) {
    return 1;
}

$options = array(
    "flags" => (string) FILTER_FLAG_ALLOW_HEX,
    "options" => array("min_range" => "0", "max_range" => "1024"),
);
var_dump(filter_var("0xff", FILTER_VALIDATE_INT, $options));
var_dump(filter_var("0xff", (string) FILTER_VALIDATE_INT, $options));
echo gettype($options["flags"]), "|", $options["options"]["min_range"], "\n";

$grouped = filter_var(
    "1,234,567,890.1234567165",
    FILTER_VALIDATE_FLOAT,
    array("flags" => FILTER_FLAG_ALLOW_THOUSAND)
);
var_dump($grouped > 1234567890 && $grouped < 1234567891);
var_dump(filter_var(
    "1234,567,890.1",
    FILTER_VALIDATE_FLOAT,
    array("flags" => FILTER_FLAG_ALLOW_THOUSAND)
));
var_dump(filter_var("1e-324", FILTER_VALIDATE_FLOAT));
var_dump(filter_var(
    "1000",
    FILTER_VALIDATE_FLOAT,
    array("options" => array("max_range" => 999.999, "default" => 0))
));

$data = array("bar" => array("fu<script>bar", "bar<script>fu"));
var_dump(filter_var($data, FILTER_CALLBACK, array("options" => "filter_cb")));
var_dump($data);
ob_start();
var_dump(filter_var_array(
    array("test" => "0xff"),
    array("test" => array(
        "filter" => (string) FILTER_VALIDATE_INT,
        "flags" => (string) FILTER_FLAG_ALLOW_HEX,
    ))
));
echo rtrim(ob_get_clean(), "\n");

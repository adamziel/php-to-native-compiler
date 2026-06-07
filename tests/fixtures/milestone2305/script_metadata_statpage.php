<?php
$checks = array(
    "lastmod" => is_int(getlastmod()) && getlastmod() > 0,
    "inode" => is_int(getmyinode()) && getmyinode() > 0,
    "uid" => is_int(getmyuid()) && getmyuid() >= 0,
    "pid" => is_int(getmypid()) && getmypid() > 0,
    "gid" => is_int(getmygid()) && getmygid() >= 0,
);
foreach ($checks as $name => $ok) {
    echo $ok ? $name : "bad-$name";
    echo "|";
}
echo function_exists("getlastmod") ? "fn" : "missing";
echo "|";
$reflection = new ReflectionFunction("getmyinode");
echo $reflection->getNumberOfRequiredParameters(), "/", $reflection->getNumberOfParameters();

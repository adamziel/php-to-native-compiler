<?php
$_SERVER = [];

function fix_server_vars() {
    $_SERVER = array_merge(['SERVER_SOFTWARE' => '', 'REQUEST_URI' => ''], $_SERVER);
    if (empty($_SERVER['REQUEST_URI'])) {
        $_SERVER['REQUEST_URI'] = '/';
    }
    $_SERVER['HTTP_HOST'] = 'example.test';
}

fix_server_vars();
echo $_SERVER['SERVER_SOFTWARE'];
echo '|';
echo $_SERVER['REQUEST_URI'];
echo '|';
echo $_SERVER['HTTP_HOST'];
echo '|';
echo PHP_SAPI;

<?php
$salt = (string) rand();
echo '{' . hash_hmac('sha256', uniqid($salt, true), $salt) . '}';

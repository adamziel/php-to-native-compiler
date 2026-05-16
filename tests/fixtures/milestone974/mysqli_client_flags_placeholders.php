<?php
$handle = mysqli_init();
mysqli_ssl_set($handle, null, null, null, null, null);
mysqli_options($handle, MYSQLI_OPT_SSL_VERIFY_SERVER_CERT, true);

$flags = MYSQLI_CLIENT_SSL
    | MYSQLI_CLIENT_FOUND_ROWS
    | MYSQLI_CLIENT_IGNORE_SPACE
    | MYSQLI_CLIENT_SSL_DONT_VERIFY_SERVER_CERT;

echo mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, $flags)
    ? "connected"
    : "failed";
echo "|";
echo MYSQLI_CLIENT_SSL, ":", MYSQLI_CLIENT_COMPRESS, ":", MYSQLI_CLIENT_INTERACTIVE;
echo ":";
echo MYSQLI_CLIENT_IGNORE_SPACE, ":", MYSQLI_CLIENT_NO_SCHEMA, ":", MYSQLI_CLIENT_FOUND_ROWS;
echo ":";
echo MYSQLI_CLIENT_SSL_VERIFY_SERVER_CERT, ":", MYSQLI_CLIENT_SSL_DONT_VERIFY_SERVER_CERT;
echo ":";
echo MYSQLI_CLIENT_CAN_HANDLE_EXPIRED_PASSWORDS;

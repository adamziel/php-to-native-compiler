<?php
class wpdb {
    public $dbh;
    public $poll_function;
    public $async_flag;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_options($this->dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->poll_function = function_exists("mysqli_poll") && is_callable("mysqli_poll");
        $this->async_flag = MYSQLI_ASYNC;
    }

    public function poll_async_boundary() {
        $read = [$this->dbh];
        $error = [];
        $reject = [];
        return mysqli_poll($read, $error, $reject, 0);
    }
}

$wpdb = new wpdb();
echo $wpdb->poll_function ? "poll-ready" : "poll-missing";
echo "\n";
echo $wpdb->async_flag;
echo "\n";
$wpdb->poll_async_boundary();

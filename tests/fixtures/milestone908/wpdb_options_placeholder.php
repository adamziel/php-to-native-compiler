<?php
class wpdb {
    public $dbh;
    public $native_types;
    public $option_checked;
    public $ready;

    public function __construct() {
        $this->dbh = mysqli_init();
        $this->native_types = false;
        $this->option_checked = false;
        $this->ready = false;
    }

    public function init_connection_options() {
        $this->native_types = mysqli_options($this->dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
        $this->option_checked = true;

        return $this->native_types;
    }

    public function connect() {
        $this->init_connection_options();
        $this->ready = mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);

        return $this->ready;
    }
}

$wpdb = new wpdb();
echo $wpdb->connect() ? "connected" : "failed";
echo "\n";
echo $wpdb->native_types ? "native-types" : "string-types";
echo "\n";
echo $wpdb->option_checked ? "checked" : "skipped";
echo "\n";
echo $wpdb->ready ? "ready" : "not-ready";

<?php
class wpdb {
    public $dbh;
    public $client_thread_safe;
    public $diagnostics_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_options($this->dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->client_thread_safe = false;
        $this->diagnostics_checked = false;
    }

    public function record_thread_safety() {
        $this->client_thread_safe = mysqli_thread_safe();
        $this->diagnostics_checked = true;

        return $this->client_thread_safe;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_thread_safety() ? "thread-safe" : "not-safe";
echo "\n";
echo $wpdb->client_thread_safe ? "recorded" : "missing";
echo "\n";
echo $wpdb->diagnostics_checked ? "checked" : "skipped";

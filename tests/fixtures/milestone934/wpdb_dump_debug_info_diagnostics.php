<?php
class wpdb {
    public $dbh;
    public $diagnostics;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_options($this->dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->diagnostics = [];
    }

    public function record_connection_diagnostics() {
        $dump = "mysqli_dump_debug_info";
        $this->diagnostics["callable"] = function_exists($dump) && is_callable($dump)
            ? "callable"
            : "missing";
        $this->diagnostics["dumped"] = $dump($this->dbh) ? "dumped" : "failed";
        $this->diagnostics["open"] = mysqli_ping($this->dbh) ? "open" : "closed";

        return $this->diagnostics["dumped"];
    }
}

$wpdb = new wpdb();
echo $wpdb->record_connection_diagnostics();
echo "\n";
echo $wpdb->diagnostics["callable"];
echo "\n";
echo $wpdb->diagnostics["dumped"];
echo "\n";
echo $wpdb->diagnostics["open"];

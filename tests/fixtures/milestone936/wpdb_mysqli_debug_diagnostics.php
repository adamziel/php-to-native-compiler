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

    public function configure_debug_trace() {
        $debug = "mysqli_debug";
        $this->diagnostics["callable"] = function_exists($debug) && is_callable($debug)
            ? "callable"
            : "missing";
        $this->diagnostics["configured"] = $debug("d:t:o,/tmp/wpdb-phpc-debug.trace")
            ? "configured"
            : "failed";
        $this->diagnostics["open"] = mysqli_ping($this->dbh) ? "open" : "closed";

        return $this->diagnostics["configured"];
    }
}

$wpdb = new wpdb();
echo $wpdb->configure_debug_trace();
echo "\n";
echo $wpdb->diagnostics["callable"];
echo "\n";
echo $wpdb->diagnostics["configured"];
echo "\n";
echo $wpdb->diagnostics["open"];

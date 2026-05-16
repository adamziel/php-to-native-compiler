<?php
class wpdb {
    public $dbh;
    public $client_stats;
    public $diagnostics_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_options($this->dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->client_stats = [];
        $this->diagnostics_checked = false;
    }

    public function record_client_stats() {
        $this->client_stats = mysqli_get_client_stats();
        $this->diagnostics_checked = true;

        return $this->client_stats["bytes_sent"] === 0
            && $this->client_stats["bytes_received"] === 0
            && $this->client_stats["packets_sent"] === 0
            && $this->client_stats["packets_received"] === 0 ? 0 : 1;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_client_stats();
echo "\n";
echo $wpdb->client_stats["bytes_sent"];
echo "\n";
echo $wpdb->client_stats["bytes_received"];
echo "\n";
echo $wpdb->client_stats["connect_success"];
echo "\n";
echo $wpdb->client_stats["active_connections"];
echo "\n";
echo $wpdb->diagnostics_checked ? "checked" : "skipped";

<?php
class wpdb {
    public $dbh;
    public $connection_stats;
    public $bytes_sent;
    public $bytes_received;
    public $connection_count;
    public $metadata_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->connection_stats = array();
        $this->bytes_sent = -1;
        $this->bytes_received = -1;
        $this->connection_count = 0;
        $this->metadata_checked = false;
    }

    public function record_connection_stats() {
        $this->connection_stats = mysqli_get_connection_stats($this->dbh);
        $this->bytes_sent = $this->connection_stats["bytes_sent"];
        $this->bytes_received = $this->connection_stats["bytes_received"];
        $this->connection_count = $this->connection_stats["active_connections"];
        $this->metadata_checked = true;

        return count($this->connection_stats);
    }
}

$wpdb = new wpdb();
echo $wpdb->record_connection_stats();
echo "\n";
echo $wpdb->bytes_sent;
echo "\n";
echo $wpdb->bytes_received;
echo "\n";
echo $wpdb->connection_count;
echo "\n";
echo $wpdb->connection_stats["connect_success"];
echo "\n";
echo $wpdb->connection_stats["result_set_queries"];
echo "\n";
echo $wpdb->metadata_checked ? "checked" : "skipped";

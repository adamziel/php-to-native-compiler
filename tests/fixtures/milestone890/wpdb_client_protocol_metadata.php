<?php
class wpdb {
    public $dbh;
    public $client_info;
    public $protocol_version;
    public $metadata_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->client_info = "";
        $this->protocol_version = 0;
        $this->metadata_checked = false;
    }

    public function record_connection_metadata() {
        $this->client_info = mysqli_get_client_info($this->dbh);
        $this->protocol_version = mysqli_get_proto_info($this->dbh);
        $this->metadata_checked = true;

        return $this->client_info;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_connection_metadata();
echo "\n";
echo $wpdb->client_info;
echo "\n";
echo $wpdb->protocol_version;
echo "\n";
echo $wpdb->metadata_checked ? "checked" : "skipped";

<?php
class wpdb {
    public $dbh;
    public $first_title;
    public $second_rows;
    public $done;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function fetch_multi_results() {
        mysqli_multi_query($this->dbh, "SELECT ID, post_title FROM wp_posts WHERE ID = 1; SELECT * FROM wp_posts WHERE 1 = 0");
        $first = mysqli_store_result($this->dbh);
        $row = mysqli_fetch_assoc($first);
        $this->first_title = $row["post_title"];
        mysqli_next_result($this->dbh);
        $second = mysqli_store_result($this->dbh);
        $this->second_rows = mysqli_num_rows($second);
        $this->done = !mysqli_more_results($this->dbh);
    }
}

$wpdb = new wpdb();
$wpdb->fetch_multi_results();
echo $wpdb->first_title, "|", $wpdb->second_rows, "|", $wpdb->done ? "done" : "more";

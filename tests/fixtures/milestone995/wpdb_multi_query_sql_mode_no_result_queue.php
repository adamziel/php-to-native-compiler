<?php
class wpdb {
    public $dbh;
    public $title;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function get_sql_mode_then_post() {
        mysqli_multi_query($this->dbh, "SELECT @@SESSION.sql_mode; SELECT ID, post_title FROM wp_posts WHERE ID = 1");
        mysqli_store_result($this->dbh);
        mysqli_next_result($this->dbh);
        $result = mysqli_store_result($this->dbh);
        $row = mysqli_fetch_assoc($result);
        $this->title = $row["post_title"];
    }
}

$wpdb = new wpdb();
$wpdb->get_sql_mode_then_post();
echo $wpdb->title;

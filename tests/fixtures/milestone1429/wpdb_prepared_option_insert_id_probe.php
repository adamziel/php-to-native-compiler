<?php
class wpdb {
    public $dbh;
    public $insert_id;

    public function __construct() {
        $this->insert_id = 0;
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);
    }

    public function insert_option($name, $value, $autoload) {
        $stmt = mysqli_prepare($this->dbh, 'INSERT INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?)');
        mysqli_stmt_bind_param($stmt, 'sss', $name, $value, $autoload);
        $ok = mysqli_stmt_execute($stmt);
        $this->insert_id = mysqli_stmt_insert_id($stmt);
        return $ok;
    }

    public function update_option_value($name, $value) {
        $stmt = mysqli_prepare($this->dbh, 'UPDATE wp_options SET option_value = ? WHERE option_name = ?');
        mysqli_stmt_execute($stmt, array($value, $name));
        return mysqli_stmt_affected_rows($stmt) . ':' . mysqli_stmt_insert_id($stmt);
    }

    public function rows() {
        return mysqli_query($this->dbh, 'SELECT option_id, option_name, option_value, autoload FROM wp_options');
    }
}

function option_parts($result) {
    $parts = array();
    while ($row = mysqli_fetch_assoc($result)) {
        $parts[] = $row['option_id'] . ':' . $row['option_name'] . '=' . $row['option_value'] . ':' . $row['autoload'];
    }
    return implode(',', $parts);
}

$wpdb = new wpdb();

echo $wpdb->insert_option('siteurl', 'https://example.test', 'yes') ? 'siteurl' : 'failed';
echo ':' . $wpdb->insert_id . ':' . mysqli_insert_id($wpdb->dbh);
echo '|';

echo $wpdb->insert_option('_transient_feed_mod', 'cached-feed', 'no') ? 'transient' : 'failed';
echo ':' . $wpdb->insert_id . ':' . mysqli_insert_id($wpdb->dbh);
echo '|';

echo $wpdb->insert_option('siteurl', 'duplicate', 'no') ? 'duplicate' : 'duplicate-rejected';
echo ':' . $wpdb->insert_id . ':' . mysqli_affected_rows($wpdb->dbh);
echo '|';

echo 'update=' . $wpdb->update_option_value('_transient_feed_mod', 'cached-new');
echo '|';

echo option_parts($wpdb->rows());

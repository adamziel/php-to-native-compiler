<?php
class wpdb {
    public $dbh;
    public $options;

    public function __construct($prefix) {
        $this->options = $prefix . 'options';
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);
    }

    public function query($query, $params) {
        return mysqli_execute_query($this->dbh, $query, $params);
    }

    public function get_row($query, $params) {
        $result = mysqli_execute_query($this->dbh, $query, $params);
        $row = mysqli_fetch_assoc($result);
        if ($row) {
            return $row;
        }
        return false;
    }
}

function option_mutation_probe($name) {
    global $wpdb;

    $insert_sql =
        'INSERT INTO `' . $wpdb->options . '` (`option_name`, `option_value`, `autoload`) ' .
        'VALUES (?, ?, ?)';
    $update_sql =
        'UPDATE `' . $wpdb->options . '` SET `option_value` = ?, `autoload` = ? ' .
        'WHERE `option_name` = ?';

    $inserted = $wpdb->query($insert_sql, array($name, 'draft', 'yes'));
    $insert_affected = mysqli_affected_rows($wpdb->dbh);
    $insert_id = mysqli_insert_id($wpdb->dbh);

    $updated = $wpdb->query($update_sql, array('published', 'auto-on', $name));
    $update_affected = mysqli_affected_rows($wpdb->dbh);

    $value_only = $wpdb->query(
        'UPDATE wp_options SET option_value = ? WHERE option_name = ?',
        array('final', $name)
    );
    $value_affected = mysqli_affected_rows($wpdb->dbh);

    $missing_update = $wpdb->query(
        'UPDATE wp_options SET autoload = ? WHERE option_name = ?',
        array('no', 'missing_option')
    );
    $missing_affected = mysqli_affected_rows($wpdb->dbh);

    $row = $wpdb->get_row(
        'SELECT option_value, autoload FROM wp_options WHERE option_name = ? LIMIT 1',
        array($name)
    );

    return
        ($inserted ? 'insert' : 'insert-failed') . ':' . $insert_affected . ':' . $insert_id .
        '|update=' . ($updated ? 'ok' : 'failed') . ':' . $update_affected .
        '|value=' . ($value_only ? 'ok' : 'failed') . ':' . $value_affected .
        '|missing=' . ($missing_update ? 'ok' : 'failed') . ':' . $missing_affected .
        '|row=' . $row['option_value'] . ':' . $row['autoload'];
}

$wpdb = new wpdb('wp_');

echo option_mutation_probe('blog_public_probe');
echo '|';

$replace_sql = 'REPLACE INTO `wp_options` (`option_name`, `option_value`, `autoload`) VALUES (?, ?, ?)';
echo $wpdb->query($replace_sql, array('blog_public_probe', 'replaced', 'no')) ? 'replace' : 'replace-failed';
echo ':' . mysqli_affected_rows($wpdb->dbh) . ':' . mysqli_insert_id($wpdb->dbh);
echo '|';

echo $wpdb->query('DELETE FROM wp_options WHERE option_name = ?', array('blog_public_probe')) ? 'delete' : 'delete-failed';
echo ':' . mysqli_affected_rows($wpdb->dbh);
echo '|';

$deleted = $wpdb->get_row(
    'SELECT option_value FROM wp_options WHERE option_name = ? LIMIT 1',
    array('blog_public_probe')
);
echo $deleted ? 'still-present' : 'gone';

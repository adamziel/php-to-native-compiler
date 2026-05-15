<?php
echo vsprintf(
    "SELECT option_value FROM wp_options WHERE option_name = '%s' LIMIT %d",
    [ "rewrite_rules", 1 ]
);
echo "\n";
echo vsprintf(
    "SELECT %% literal, `%s`, '%s', %05d, %.2F",
    [ "wp_posts", "post_name", "7", 3.5 ]
);

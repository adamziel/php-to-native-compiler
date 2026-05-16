<?php
$handle = mysqli_init();
mysqli_query($handle, "LOAD DATA LOCAL INFILE '/tmp/posts.csv' INTO TABLE wp_posts");

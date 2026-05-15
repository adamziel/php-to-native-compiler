<?php
$handle = mysqli_init();
mysqli_query($handle, "UPDATE wp_options SET option_value = '1' WHERE option_name = 'blog_public'");

<?php

ob_start();

function dump_json_location($json, $depth = 512) {
    var_dump(json_validate($json, $depth));
    var_dump(json_last_error(), json_last_error_msg());
}

dump_json_location("{
    \"name\": \"value
}");
dump_json_location('{"val": tru}');
dump_json_location('{"key": "\q"}');
dump_json_location('["val"}');
dump_json_location('[[[[[[10]]]]]]', 5);
dump_json_location('{"\u30D7\u30EC\u30B9": "value}');
dump_json_location("  \t  \n  ");
dump_json_location('{"num": 1e}');
dump_json_location('{"num": --1}');

echo rtrim(ob_get_clean(), "\n");

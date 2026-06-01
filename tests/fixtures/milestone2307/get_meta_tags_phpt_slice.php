<?php
$filename = __DIR__ . "/get_meta_tags_fixture.html";
$array = array(
    "<meta name=\"author\" content=\"name\">\n<meta name=\"keywords\" content=\"php documentation\">\n<meta name=\"DESCRIPTION\" content=\"a php manual\">\n<meta name=\"geo.position\" content=\"49.33;-86.59\">\n</head> <!-- parsing stops here -->",
    "<html>\n    <head>\n        <meta name=\"author\" content=\"name\">\n        <meta name=\"keywords\" content=\"php documentation\">\n        <meta name=\"DESCRIPTION\" content=\"a php manual\">\n        <meta name=\"geo.position\" content=\"49.33;-86.59\">\n    </head>\n    <body>\n        <meta name=\"author\" content=\"name1\">\n        <meta name=\"keywords\" content=\"php documentation1\">\n    </body>\n</html>",
    "<meta name=\"author\" content=\"name\"\n<meta name=\"keywords\" content=\"php documentation\">",
    "<meta <meta name=\"keywords\" content=\"php documentation\">",
    "<meta name=\"author\" content=\"name\"\n<meta name=\"keywords\" content=\"php documentation\"",
    "",
    "<>",
    "<meta<<<<<"
);
foreach ($array as $html) {
    file_put_contents($filename, $html);
    var_dump(get_meta_tags($filename));
}
unlink($filename);
echo "Done";

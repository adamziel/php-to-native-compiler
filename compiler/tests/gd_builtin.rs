use php_compiler::run_source;

#[test]
fn gd_bounded_image_handles_and_error_surfaces_are_available() {
    let execution = run_source(
        r#"<?php
var_dump(defined("IMG_FILTER_COLORIZE"));

$true = imagecreatetruecolor(180, 30);
var_dump($true instanceof GdImage);
var_dump(imageantialias($true, "wrong param"));
var_dump(imagefilter($true, IMG_FILTER_COLORIZE, 800, 255, 255));

try {
    imagefilter($true);
} catch (TypeError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    imagefilter(20, 1);
} catch (TypeError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    imagefilter($true, -1);
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}

$palette = imagecreate(180, 30);
$white = imagecolorallocate($palette, 255, 255, 255);
echo $white, "|", imagecolorstotal($palette), "\n";
try {
    imagecolordeallocate($palette, imagecolorstotal($palette) + 100);
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    imagecolorset($palette, $white, -3, 4, 5, 6);
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "TypeError:imagefilter() expects at least 2 arguments, 1 given\n",
            "TypeError:imagefilter(): Argument #1 ($image) must be of type GdImage, int given\n",
            "ValueError:imagefilter(): Argument #2 ($filter) must be one of the IMG_FILTER_* filter constants\n",
            "0|1\n",
            "ValueError:imagecolordeallocate(): Argument #2 ($color) must be between 0 and 1\n",
            "ValueError:imagecolorset(): Argument #3 ($red) must be between 0 and 255 (inclusive)\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn gd_colormatch_and_convolution_validate_image_shape_and_matrix_keys() {
    let execution = run_source(
        r#"<?php
$palette1 = imagecreate(110, 20);
$palette2 = imagecreate(110, 20);
try {
    imagecolormatch($palette1, $palette2);
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}

$true1 = imagecreatetruecolor(110, 20);
$true2 = imagecreatetruecolor(110, 20);
try {
    imagecolormatch($true1, $true2);
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}

$palette_small = imagecreate(100, 20);
try {
    imagecolormatch($true1, $palette_small);
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}

$bad_size = [
    [1.0, 2.0, 1.0],
    [2.0, 4.0, 2.0],
];
try {
    imageconvolution($true1, $bad_size, 16, 0);
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}

$bad_key = [
    [1.0, 2.0, 1.0],
    [2.0, 4.0, 2.0],
    [1.0, 2.0, "x" => 1.0],
];
try {
    imageconvolution($true1, $bad_key, 16, 0);
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "ValueError:imagecolormatch(): Argument #1 ($image1) must be TrueColor\n",
            "ValueError:imagecolormatch(): Argument #2 ($image2) must be Palette\n",
            "ValueError:imagecolormatch(): Argument #2 ($image2) must be the same size as argument #1 ($im1)\n",
            "ValueError:imageconvolution(): Argument #2 ($matrix) must be a 3x3 array\n",
            "ValueError:imageconvolution(): Argument #2 ($matrix) must be a 3x3 array, matrix[2][2] cannot be found (missing integer key)\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

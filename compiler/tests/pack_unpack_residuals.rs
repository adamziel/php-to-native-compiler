use php_compiler::run_source;

#[test]
fn pack_unpack_little_endian_arrays_and_offsets() {
    let execution = run_source(
        r#"<?php
$str = pack('VVV', 0x00010203, 0x04050607, 0x08090a0b);
print_r(unpack('Vaa/Vbb/Vcc', $str));
print_r(unpack('V2aa/Vcc', $str));
print_r(unpack('V3aa', $str));
print_r(unpack('V*aa', $str));
print_r(unpack('V*', $str));

$data = "pad" . pack("ll", 0x01020304, 0x05060708);
$a = unpack("l2", $data, 3);
printf("0x%08x 0x%08x\n", $a[1], $a[2]);
printf("0x%08x 0x%08x\n", unpack("l", $data, 3)[1], unpack("@4/l", $data, 3)[1]);
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains("[aa] => 66051"));
    assert!(execution.stdout.contains("[bb] => 67438087"));
    assert!(execution.stdout.contains("[aa3] => 134810123"));
    assert!(execution.stdout.contains("[3] => 134810123"));
    assert!(execution
        .stdout
        .contains("0x01020304 0x05060708\n0x01020304 0x05060708"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn pack_signed_char_writes_low_byte_values() {
    let execution = run_source(
        r#"<?php
echo bin2hex(pack("c", -1)), "\n";
echo bin2hex(pack("c", 0)), "\n";
echo bin2hex(pack("c", 127)), "\n";
echo bin2hex(pack("c", 128)), "\n";
echo bin2hex(pack("c", 255)), "\n";
echo bin2hex(pack("c", 256)), "\n";
echo base64_encode(pack("c", 255)), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "ff\n00\n7f\n80\nff\n00\n/w==\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn pack_unpack_string_fields_and_cursor_controls() {
    let execution = run_source(
        r#"<?php
echo bin2hex(pack("A5", "foo ")), "\n";
echo bin2hex(pack("Z4", "fooo")), "\n";
var_dump(unpack("A*", "foo\0\rbar\0 \t\r\n"));
var_dump(unpack("A4", "foo\0\rbar\0 \t\r\n"));
var_dump(unpack("Z*", "foo\0\rbar\0 \t\r\n"));
var_dump(unpack("Z2", "AB\0"));

$data = pack('VV', 1, 2);
var_dump(unpack('Va/X', $data));
var_dump(unpack('Va/X4', $data));
var_dump(unpack('V1a/X4/V1b/V1c/X4/V1d', $data));
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains("666f6f2020\n"));
    assert!(execution.stdout.contains("666f6f00\n"));
    assert!(execution.stdout.contains("string(8) \"foo\0\rbar\""));
    assert!(execution.stdout.contains("string(3) \"foo\""));
    assert!(execution.stdout.contains("string(2) \"AB\""));
    assert!(execution.stdout.contains("[\"a\"]=>\n  int(1)"));
    assert!(execution.stdout.contains("[\"d\"]=>\n  int(2)"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn pack_unpack_float_and_64bit_numeric_codes() {
    let execution = run_source(
        r#"<?php
echo bin2hex(pack("e", 1.0)), "\n";
echo bin2hex(pack("E", 1.0)), "\n";
echo bin2hex(pack("g", 1.0)), "\n";
echo bin2hex(pack("G", 1.0)), "\n";
var_dump(unpack("e", hex2bin("000000000000f03f")));
var_dump(unpack("E", hex2bin("3ff0000000000000")));
var_dump(unpack("g", hex2bin("0000803f")));
var_dump(unpack("G", hex2bin("3f800000")));
print_r(unpack("Q", pack("Q", 0xfffffffffffe)));
print_r(unpack("J", pack("J", 0xfffffffffffe)));
print_r(unpack("P", pack("P", 0xfffffffffffe)));
print_r(unpack("q", pack("q", -1)));
print_r(unpack("I", pack("I", 4294967295)));
print_r(unpack("Q", pack("Q", 0x8000000000000002)));
	"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("000000000000f03f\n3ff0000000000000\n"));
    assert!(execution.stdout.contains("0000803f\n3f800000\n"));
    assert!(execution.stdout.matches("float(1)").count() >= 4);
    assert!(execution.stdout.contains("[1] => 281474976710654"));
    assert!(execution.stdout.contains("[1] => -1"));
    assert!(execution.stdout.contains("[1] => 4294967295"));
    assert!(execution.stdout.contains("not representable as an int"));
    assert!(execution.stdout.contains("[1] => -9223372036854775808"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unpack_errors_match_value_error_boundaries() {
    let execution = run_source(
        r#"<?php
try {
    var_dump(unpack("B", pack("I", 65534)));
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
foreach ([10, -1] as $offset) {
    try {
        unpack("l", "foo", $offset);
    } catch (ValueError $e) {
        echo $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Invalid format type B\n\
unpack(): Argument #3 ($offset) must be contained in argument #2 ($data)\n\
unpack(): Argument #3 ($offset) must be contained in argument #2 ($data)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn pack_unpack_cover_star_cursor_warning_and_too_few_valueerror() {
    let execution = run_source(
        r#"<?php
var_dump(unpack("X*", ""));
try {
    var_dump(pack("E2E2147483647H*", 0x0, 0x0, 0x0));
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert!(
        execution
            .stdout
            .contains("Warning: unpack(): Type X: '*' ignored"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("array(0) {\n}"),
        "{}",
        execution.stdout
    );
    assert!(execution.stdout.ends_with("Type E: too few arguments\n"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn pack_unpack_declared_operands_use_php_argument_boundaries() {
    let execution = run_source(
        r#"<?php
class FormatValue {
    public function __toString(): string {
        return "H*";
    }
}
class DataValue {
    public function __toString(): string {
        return "4142";
    }
}

echo bin2hex(pack(new FormatValue, "4142")), "|";
$unpacked = unpack(new FormatValue, new DataValue);
echo $unpacked[1], "|";
$call = "unpack";
$offset = $call("H*", new DataValue, "1");
echo $offset[1], "|";
echo strlen(pack("", "unused")), ":", count(unpack("", "AB")), "|";

set_error_handler(function($_errno, $message) {
    echo $message, "|";
    return true;
});
$empty = unpack("H*", null);
echo count($empty), ":", $empty[1], "|";
restore_error_handler();

try {
    pack([], "41");
} catch (TypeError $e) {
    echo $e->getMessage(), "|";
}
try {
    unpack("H*", []);
} catch (TypeError $e) {
    echo $e->getMessage(), "|";
}
try {
    unpack("H*", "AB", []);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "4142|34313432|313432|0:0|unpack(): Passing null to parameter #2 ($string) of type string is deprecated|1:|pack(): Argument #1 ($format) must be of type string, array given|unpack(): Argument #2 ($string) must be of type string, array given|unpack(): Argument #3 ($offset) must be of type int, array given"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

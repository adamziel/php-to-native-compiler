use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn temp_hash_path(label: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    std::env::temp_dir()
        .join(format!("phpc-hash-{label}-{nanos}.txt"))
        .display()
        .to_string()
}

#[test]
fn hash_hmac_sha256_and_uniqid_cover_current_placeholder_slice() {
    let execution = run_source(
        r#"<?php
echo function_exists("hash_hmac") ? "hash" : "missing";
echo "|";
echo is_callable("uniqid") ? "uniqid" : "missing";
echo "|";
echo uniqid("salt", true);
echo "|";
echo hash_hmac("sha256", "data", "key");
echo "|";
$call = "hash_hmac";
echo $call("sha256", uniqid("salt", true), "salt", false);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "hash|uniqid|salt0000000000000.000000000|5031fe3d989c6d1537a013fa6e739da23463fdaec3b70137d828e36ace221bd0|a7afc0875fafe2923927b020bdb1a243deb47ffd9d2429731c87e88dc3cf2d4f"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn uniqid_more_entropy_matches_php_shaped_lengths() {
    let execution = run_source(
        r#"<?php
echo strlen(uniqid()), ":", uniqid(), "\n";
echo strlen(uniqid('', true)), ":", uniqid('', true), "\n";
echo strlen(uniqid(99999, true)), ":", uniqid(99999, true), "\n";
echo strlen(uniqid(10.5e2, true)), ":", uniqid(10.5e2, true), "\n";
echo strlen(uniqid(true, true)), ":", uniqid(true, true), "\n";
echo strlen(uniqid(false, true)), ":", uniqid(false, true), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "13:0000000000000\n23:0000000000000.000000000\n28:999990000000000000.000000000\n27:10500000000000000.000000000\n24:10000000000000.000000000\n23:0000000000000.000000000\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_hmac_and_uniqid_reject_forms_outside_current_boundary() {
    let tiger4 =
        run_source("<?php\necho hash_hmac('tiger128,4', 'data', 'key'), \"\\n\";\n").unwrap();
    assert_eq!(tiger4.stdout, "cc2c6d31a589cab47f09390df815fe0d\n");

    let raw = run_source(
        r#"<?php
try {
    hash_hmac('sha256', 'data', 'key', []);
} catch (Error $e) {
    echo get_class($e), ":", $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        raw.stdout,
        "TypeError:hash_hmac(): Argument #4 ($binary) must be of type bool, array given"
    );

    let entropy = run_source("<?php\nuniqid('salt', 1);\n").unwrap_err();
    assert_eq!(entropy.phase, Phase::Runtime);
    assert_eq!(entropy.line, 2);
    assert_eq!(entropy.column, 1);
    assert_eq!(
        entropy.message,
        "unsupported call uniqid(): more_entropy argument must be bool in the current subset, got int"
    );
}

#[test]
fn hash_sha_family_and_algorithm_listing_are_available() {
    let execution = run_source(
        r#"<?php
echo hash("sha1", "abc"), "\n";
echo hash("sha224", "abc"), "\n";
echo hash("sha256", "abc"), "\n";
echo hash("sha384", "abc"), "\n";
echo hash("sha512/224", "abc"), "\n";
echo hash("sha512/256", "abc"), "\n";
echo substr(hash("sha512", "abc"), 0, 16), "\n";
echo bin2hex(hash("sha256", "abc", true)), "\n";
echo in_array("sha512/256", hash_algos(), true) ? "listed" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "a9993e364706816aba3e25717850c26c9cd0d89d\n\
23097d223405d8228642a477bda255b32aadbce4bda0b3f7e36c9da7\n\
ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\n\
cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7\n\
4634270f707b6a54daae7530460842e20e37ed265ceee9a43e8924aa\n\
53048e2681941ef99b2e29b76b4c7dabe4c2d0c634fc6d46e0e2f13107e7af23\n\
ddaf35a193617aba\n\
ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\n\
listed"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_md5_and_small_pure_digest_algorithms_are_available() {
    let execution = run_source(
        r#"<?php
echo hash("md5", ""), "\n";
echo hash("md5", "a"), "\n";
echo hash("md5", "012345678901234567890123456789012345678901234567890123456789"), "\n";
var_dump(hash("md5", "string") === md5("string"));
echo bin2hex(hash("md5", "string", true)), "\n";
echo hash("adler32", ""), "\n";
echo hash("adler32", "abc"), "\n";
echo hash("fnv132", ""), "\n";
echo hash("fnv132", "foobar"), "\n";
echo bin2hex(hash("fnv132", "", true)), "\n";
echo hash("fnv1a32", "a"), "\n";
echo hash("fnv164", ""), "\n";
echo hash("fnv164", "foobar"), "\n";
echo hash("fnv1a64", "9"), "\n";
echo hash("joaat", "hello world"), "\n";
echo bin2hex(hash("joaat", "", true)), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "d41d8cd98f00b204e9800998ecf8427e\n\
0cc175b9c0f1b6a831c399e269772661\n\
1ced811af47ead374872fcca9d73dd71\n\
bool(true)\n\
b45cffe084dd3d20d928bee85e7b0f21\n\
00000001\n\
024d0127\n\
811c9dc5\n\
31f0b262\n\
811c9dc5\n\
e40c292c\n\
cbf29ce484222325\n\
340d8765a4dda9c2\n\
af63b44c8601a894\n\
3e4a5a57\n\
00000000\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_md2_and_md4_legacy_algorithms_are_available() {
    let execution = run_source(
        r#"<?php
$subjects = [
    "",
    "a",
    "abc",
    "message digest",
    "abcdefghijklmnopqrstuvwxyz",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
    "12345678901234567890123456789012345678901234567890123456789012345678901234567890",
];
foreach ($subjects as $subject) {
    echo hash("md2", $subject), "\n";
}
echo bin2hex(hash("md2", "abc", true)), "\n";
foreach ($subjects as $subject) {
    echo hash("md4", $subject), "\n";
}
echo bin2hex(hash("md4", "abc", true)), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "8350e5a3e24c153df2275c9f80692773\n\
32ec01ec4a6dac72c0ab96fb34c0b5d1\n\
da853b0d3f88d99b30283a69e6ded6bb\n\
ab4f496bfb2a530b219ff33031fe06b0\n\
4e8ddff3650292ab5a4108c3aa47940b\n\
da33def2a42df13975352846c30338cd\n\
d5976f79d83d3a0dc9806c3c66f3efd8\n\
da853b0d3f88d99b30283a69e6ded6bb\n\
31d6cfe0d16ae931b73c59d7e0c089c0\n\
bde52cb31de33e46245e05fbdbd6fb24\n\
a448017aaf21d8525fc10ae87aa6729d\n\
d9130a8164549fe818874806e1c7014b\n\
d79e1c308aa5bbcdeea8ed63df412da9\n\
043f8582f241db351ce627e153e7f0e4\n\
e33b4ddc9c38f2199c3e7b164fcc0536\n\
a448017aaf21d8525fc10ae87aa6729d\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_ripemd_and_tiger3_algorithms_are_available() {
    let execution = run_source(
        r#"<?php
echo hash("ripemd128", ""), "\n";
echo hash("ripemd160", "abc"), "\n";
echo hash("ripemd256", str_repeat("a", 1000000)), "\n";
echo hash("ripemd320", "message digest"), "\n";
echo hash("tiger192,3", ""), "\n";
echo hash("tiger192,3", str_repeat("abc", 64)), "\n";
echo bin2hex(hash("ripemd160", "abc", true)), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "cdf26213a150dc3ecb610f18f6b38b46\n\
8eb208f7e05d987a9b044a8e98c6b087f15a0bfc\n\
ac953744e10e31514c150d4d8d7b677342e33399788296e43ae4850ce4f97978\n\
3a8e28502ed45d422f68844f9dd316e7b98533fa3f2a91d29f84d425c88d6b4eff727df66a7c0197\n\
3293ac630c13f0245f92bbb1766e16167a4e58492dde73f3\n\
badd965340a9e83e4a16f48a5038c01b856a9158ef59fec1\n\
8eb208f7e05d987a9b044a8e98c6b087f15a0bfc\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_gost_haval_and_snefru_legacy_algorithms_are_available() {
    let path = temp_hash_path("legacy");
    fs::write(&path, b"abc").unwrap();

    let source = format!(
        r#"<?php
echo hash("gost", ""), "\n";
echo hash("gost-crypto", ""), "\n";
echo hash("haval128,3", ""), "\n";
echo hash("haval160,4", "abc"), "\n";
echo hash("haval256,5", "abc"), "\n";
echo hash("snefru", ""), "\n";
echo hash("snefru256", ""), "\n";

$ctx = hash_init("haval128,3");
hash_update($ctx, "a");
hash_update($ctx, "bc");
echo hash_final($ctx), "\n";

$ctx = hash_init("snefru", HASH_HMAC, "secret");
hash_update($ctx, "abc");
echo hash_final($ctx), "\n";

$content = "This is a sample string used to test the hash_hmac function with various hashing algorithms";
echo hash_hmac("gost", $content, "secret"), "\n";
echo hash_hmac("haval128,3", $content, "secret"), "\n";
echo hash_hmac("snefru", $content, "secret"), "\n";
echo hash_pbkdf2("haval128,3", "password", "salt", 1, 32), "\n";
echo bin2hex(hash_hkdf("gost", "input key material", 16)), "\n";

$file = {path:?};
echo hash_file("gost", $file), "\n";
echo hash_file("haval128,3", $file), "\n";
echo hash_file("snefru", $file), "\n";
"#
    );

    let execution = run_source(&source).unwrap();
    assert_eq!(
        execution.stdout,
        "ce85b99cc46752fffee35cab9a7b0278abb4c2d2055cff685af4912c49490f8d\n\
981e5f3ca30c841487830f84fb433e13ac1101569b9c13584ac483234cd656c0\n\
c68f39913f901f3ddf44c707357a7d70\n\
77aca22f5b12cc09010afc9c0797308638b1cb9b\n\
976cd6254c337969e5913b158392a2921af16fca51f5601d486e0a9de01156e7\n\
8617f366566a011837f4fb4ba5bedea2b892f3ed8b894023d16ae344b2be5881\n\
8617f366566a011837f4fb4ba5bedea2b892f3ed8b894023d16ae344b2be5881\n\
9e40ed883fb63e985d299b40cda2b8f2\n\
60261f1d20a9aed3d59d834eb096852f589ab11cc9f9f3577daae2ed71b48bf1\n\
a4a3c80bdf3f8665bf07376a34dc9c1b11af7c813f4928f62e39f0c0dc564dad\n\
4d1318607f0406bd1b7bd50907772672\n\
67af483046f9cf16fe19f9087929ccfc6ad176ade3290b4d33f43e0ddb07e711\n\
febde9e1d3af32109aa95d6751782de1\n\
64edd584b87a2dfdd1f2b44ed2db8bd2\n\
f3134348c44fb1b2a277729e2285ebb5cb5e0f29c975bc753b70497c06a4d51d\n\
9e40ed883fb63e985d299b40cda2b8f2\n\
7d033205647a2af3dc8339f6cb25643c33ebc622d32979c4b612b02c4903031b\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(path);
}

#[test]
fn hash_crc_algorithms_and_php82_algorithm_listing_are_available() {
    let execution = run_source(
        r#"<?php
echo hash("crc32", "123456789"), "\n";
echo hash("crc32b", "123456789"), "\n";
echo hash("crc32c", "123456789"), "\n";
echo bin2hex(hash("crc32", "123456789", true)), "\n";
$algos = hash_algos();
echo count($algos), "|", $algos[30], "|", $algos[31], "|", $algos[32];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "181989fc\ncbf43926\ne3069283\n181989fc\n60|crc32|crc32b|crc32c"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_sha3_family_is_available() {
    let execution = run_source(
        r#"<?php
foreach (["", "a", "The quick brown fox jumps over the lazy dog"] as $subject) {
    echo hash("sha3-224", $subject), "\n";
    echo hash("sha3-256", $subject), "\n";
    echo substr(hash("sha3-384", $subject), 0, 24), "\n";
    echo substr(hash("sha3-512", $subject), 0, 24), "\n";
}
echo bin2hex(hash("sha3-256", "abc", true)), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "6b4e03423667dbb73b6e15454f0eb1abd4597f9a1b078e3f5b5a6bc7\n\
a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a\n\
0c63a75b845e4f7d01107d85\n\
a69f73cca23a9ac5c8b567dc\n\
9e86ff69557ca95f405f081269685b38e3a819b309ee942f482b6a8b\n\
80084bf2fba02475726feb2cab2d8215eab14bc6bdd8bfb2c8151257032ecd8b\n\
1815f774f320491b48569efe\n\
697f2d856172cb8309d6b8b9\n\
d15dadceaa4d5d7bb3b48f446421d542e08ad8887305e28d58335795\n\
69070dda01975c8c120c3aada1b282394e7f032fa9cf32f4cb2259a0897dfc04\n\
7063465e08a93bce31cd89d2\n\
01dedd5de4ef14642445ba5f\n\
3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_whirlpool_algorithm_is_available() {
    let execution = run_source(
        r#"<?php
$subject = "---qwertzuiopasdfghjklyxcvbnm------qwertzuiopasdfghjklyxcvbnm---";
echo hash("whirlpool", ""), "\n";
echo hash("whirlpool", $subject), "\n";
echo substr(hash("whirlpool", str_repeat($subject . "0", 1000)), 0, 32), "\n";
echo bin2hex(hash("whirlpool", "abc", true)), "\n";
echo in_array("whirlpool", hash_algos(), true) ? "listed" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "19fa61d75522a4669b44e39c1d2e1726c530232130d407f89afee0964997f7a73e83be698b288febcf88e3e03c4f0757ea8964e59b63d93708b138cc42a66eb3\n\
916ce6431d2f384be68d96bcaba800c21b82e9cc2f07076554c9557f85476b5d8f2b263951121fa955e34b31a4cdc857bdf076b123c2252543dcef34f84a7ef3\n\
b51984710d11893ac08e10529519f980\n\
4e2448a4c6f486bb16b6562c73b4020bf3043e3a731bce721ae1b303d97e6d4c7181eebdb6c57e277d0e34957114cbd6c797fc9d95d8b582d225292076d4eef5\n\
listed"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_rejects_unknown_algorithms() {
    let execution = run_source("<?php\nhash('foo', '');\n").unwrap();
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 255);
    assert!(
        execution.stdout.contains(
            "Fatal error: Uncaught ValueError: hash(): Argument #1 ($algo) must be a valid hashing algorithm"
        ),
        "{}",
        execution.stdout
    );
}

#[test]
fn hash_unknown_algorithm_is_catchable_value_error() {
    let execution = run_source(
        r#"<?php
try {
    hash("foo", "");
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ValueError:hash(): Argument #1 ($algo) must be a valid hashing algorithm"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_hmac_invalid_or_non_crypto_algorithms_are_catchable_value_errors() {
    let execution = run_source(
        r#"<?php
foreach (["foo", "crc32"] as $algorithm) {
    try {
        hash_hmac($algorithm, "data", "key");
    } catch (ValueError $e) {
        echo $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "hash_hmac(): Argument #1 ($algo) must be a valid cryptographic hashing algorithm\n\
hash_hmac(): Argument #1 ($algo) must be a valid cryptographic hashing algorithm\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_init_validates_algorithm_hmac_mode_and_key_before_streaming_boundary() {
    let execution = run_source(
        r#"<?php
echo defined("HASH_HMAC") ? HASH_HMAC : "missing";
echo "|", function_exists("hash_init") ? "fn" : "missing";
echo "|", is_callable("hash_init") ? "callable" : "missing", "\n";

foreach ([
    fn() => hash_init("dummy"),
    fn() => hash_init("crc32", HASH_HMAC),
    fn() => hash_init("md5", HASH_HMAC),
    fn() => hash_init("md5", HASH_HMAC, null),
] as $test) {
    try {
        var_dump($test());
    } catch (\Error $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert!(execution.stdout.starts_with(
        "1|fn|callable\n\
ValueError:hash_init(): Argument #1 ($algo) must be a valid hashing algorithm\n\
ValueError:hash_init(): Argument #1 ($algo) must be a cryptographic hashing algorithm if HMAC is requested\n\
ValueError:hash_init(): Argument #3 ($key) must not be empty when HMAC is requested\n"
    ));
    assert!(
        execution.stdout.contains(
            "Deprecated: hash_init(): Passing null to parameter #3 ($key) of type string is deprecated"
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.ends_with(
            "ValueError:hash_init(): Argument #3 ($key) must not be empty when HMAC is requested\n"
        ),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);

    let tiger4 = run_source(
        "<?php\n$ctx = hash_init('tiger128,4'); echo $ctx instanceof HashContext ? 'ctx' : 'bad';\n",
    )
    .unwrap();
    assert_eq!(tiger4.stdout, "ctx");
}

#[test]
fn hash_murmur3_and_xxhash_options_cover_seeded_context_rows() {
    let execution = run_source(
        r#"<?php
echo hash("murmur3a", "foo"), "\n";
echo hash("murmur3c", "Two hashes meet in a bar"), "\n";
echo hash("murmur3c", "hash me!"), "\n";
echo hash("murmur3f", "Two hashes meet in a bar"), "\n";
echo hash("murmur3f", "hash me!"), "\n";

foreach (["murmur3a", "murmur3c", "murmur3f"] as $algo) {
    $ctx = hash_init($algo);
    hash_update($ctx, "hello");
    hash_update($ctx, " there");
    echo hash_final($ctx), " ", hash($algo, "hello there"), "\n";
}

$ctx = hash_init("murmur3f", options: ["seed" => 42]);
foreach (["Two", " hashes", " meet", " in", " a", " bar."] as $chunk) {
    hash_update($ctx, $chunk);
}
echo hash_final($ctx), "\n";
echo hash("murmur3f", "Two hashes meet in a bar.", options: ["seed" => 42]), "\n";
echo hash("murmur3c", "Two hashes meet in a bar.", options: ["seed" => 106]), "\n";
echo hash("murmur3a", "Two hashes meet in a bar.", options: ["seed" => 2345]), "\n";

$data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
foreach (["xxh32", "xxh64", "xxh3", "xxh128"] as $algo) {
    $ctx = hash_init($algo, options: ["seed" => 42]);
    foreach (["Lorem", " ipsum dolor", " sit amet,", " consectetur adipiscing elit."] as $chunk) {
        hash_update($ctx, $chunk);
    }
    echo hash_final($ctx), "\n";
    echo hash($algo, $data, options: ["seed" => 42]), "\n";
}

$secret = str_repeat("a", 256);
foreach (["xxh3", "xxh128"] as $algo) {
    $ctx = hash_init($algo, options: ["secret" => $secret]);
    foreach (["Lorem", " ipsum dolor", " sit amet,", " consectetur adipiscing elit."] as $chunk) {
        hash_update($ctx, $chunk);
    }
    echo hash_final($ctx), " ", hash($algo, $data, options: ["secret" => $secret]), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "f6a5c420\n\
8036c2707453c6f37348142be7eaf75c\n\
c7009299985a5627a9280372a9280372\n\
40256ed26fa6ece7785092ed33c8b659\n\
c43668294e89db0ba5772846e5804467\n\
6440964d 6440964d\n\
2bcadca212d62deb69712a721e593089 2bcadca212d62deb69712a721e593089\n\
81514cc240f57a165c95eb63f9c0eedf 81514cc240f57a165c95eb63f9c0eedf\n\
95855f9be0db784a5c37e878c4a4dcee\n\
95855f9be0db784a5c37e878c4a4dcee\n\
f64c9eb40287fa686575163893e283b2\n\
7f7ec59b\n\
3d0cc7e5\n\
3d0cc7e5\n\
9c9aa071b5d22a15\n\
9c9aa071b5d22a15\n\
366409913c16b70d\n\
366409913c16b70d\n\
f87856a7589354e92aeca886c71ed7fb\n\
f87856a7589354e92aeca886c71ed7fb\n\
8028aa834c03557a 8028aa834c03557a\n\
54279097795e7218093a05d4d781cbb9 54279097795e7218093a05d4d781cbb9\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_xxhash_option_deprecations_and_secret_errors_are_php_shaped() {
    let execution = run_source(
        r#"<?php
set_error_handler(function($_, $message) {
    echo "deprecated:", $message, "\n";
    return true;
});

foreach (["murmur3a", "murmur3c", "murmur3f", "xxh32", "xxh64", "xxh3", "xxh128"] as $algo) {
    hash_init($algo, options: ["seed" => "42"]);
}

class StringableThrowingClass {
    public function __toString(): string {
        throw new Exception("exception in __toString");
        return "";
    }
}

foreach (["xxh3", "xxh128"] as $algo) {
    try {
        hash_init($algo, options: ["seed" => 24, "secret" => str_repeat("a", 256)]);
    } catch (Throwable $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
    try {
        hash_init($algo, options: ["secret" => new StringableThrowingClass()]);
    } catch (Throwable $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
    try {
        hash_init($algo, options: ["secret" => str_repeat("a", 17)]);
    } catch (Throwable $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
    try {
        hash_init($algo, options: ["secret" => 42]);
    } catch (Throwable $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "deprecated:hash_init(): Passing a seed of a type other than int is deprecated because it is the same as setting the seed to 0\n\
deprecated:hash_init(): Passing a seed of a type other than int is deprecated because it is the same as setting the seed to 0\n\
deprecated:hash_init(): Passing a seed of a type other than int is deprecated because it is the same as setting the seed to 0\n\
deprecated:hash_init(): Passing a seed of a type other than int is deprecated because it is the same as setting the seed to 0\n\
deprecated:hash_init(): Passing a seed of a type other than int is deprecated because it is the same as setting the seed to 0\n\
deprecated:hash_init(): Passing a seed of a type other than int is deprecated because it is ignored\n\
deprecated:hash_init(): Passing a seed of a type other than int is deprecated because it is ignored\n\
Error:xxh3: Only one of seed or secret is to be passed for initialization\n\
deprecated:hash_init(): Passing a secret of a type other than string is deprecated because it implicitly converts to a string, potentially hiding bugs\n\
Exception:exception in __toString\n\
Error:xxh3: Secret length must be >= 136 bytes, 17 bytes passed\n\
deprecated:hash_init(): Passing a secret of a type other than string is deprecated because it implicitly converts to a string, potentially hiding bugs\n\
Error:xxh3: Secret length must be >= 136 bytes, 2 bytes passed\n\
Error:xxh128: Only one of seed or secret is to be passed for initialization\n\
deprecated:hash_init(): Passing a secret of a type other than string is deprecated because it implicitly converts to a string, potentially hiding bugs\n\
Exception:exception in __toString\n\
Error:xxh128: Secret length must be >= 136 bytes, 17 bytes passed\n\
deprecated:hash_init(): Passing a secret of a type other than string is deprecated because it implicitly converts to a string, potentially hiding bugs\n\
Error:xxh128: Secret length must be >= 136 bytes, 2 bytes passed\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_context_streaming_and_file_paths_cover_bounded_rows() {
    let path = temp_hash_path("streaming");
    fs::write(&path, b"abc").unwrap();

    let source = format!(
        r#"<?php
$file = {path:?};
foreach (["hash_update", "hash_final", "hash_copy", "hash_file", "hash_update_file", "hash_update_stream"] as $name) {{
    echo function_exists($name) && is_callable($name) ? "1" : "0";
}}
echo "\n";

$context = hash_init("md5");
echo $context instanceof HashContext ? "ctx" : "bad", "\n";
var_dump(hash_update($context, "a"));
$copy = hash_copy($context);
hash_update($context, "bc");
hash_update($copy, "-copy");
echo hash_final($context), "\n";
echo hash_final($copy), "\n";

echo hash_file("md5", $file), "|";
echo hash_file("sha1", $file), "|";
echo hash_file("sha256", $file), "|";
echo strlen(hash_file("md5", $file, true)), "\n";

$context = hash_init("md5");
var_dump(hash_update_file($context, $file));
echo hash_final($context), "\n";

$stream = tmpfile();
fwrite($stream, "abc");
rewind($stream);
$context = hash_init("md5");
echo hash_update_stream($context, $stream), "|", hash_final($context), "\n";

$stream = tmpfile();
fwrite($stream, "abc");
rewind($stream);
$context = hash_init("md5");
echo hash_update_stream($context, $stream, 0), "|", hash_final($context), "\n";

$context = hash_init("sha1");
hash_final($context);
foreach ([fn() => hash_update($context, "x"), fn() => hash_final($context), fn() => hash_copy($context)] as $test) {{
    try {{
        var_dump($test());
    }} catch (\Error $e) {{
        echo get_class($e), ":", $e->getMessage(), "\n";
    }}
}}

try {{
    hash_file("not-real", $file);
}} catch (\Error $e) {{
    echo get_class($e), ":", $e->getMessage(), "\n";
}}

try {{
    new HashContext();
}} catch (\Error $e) {{
    echo get_class($e), ":", $e->getMessage(), "\n";
}}
"#
    );

    let execution = run_source(&source).unwrap();
    assert_eq!(
        execution.stdout,
        "111111\n\
ctx\n\
bool(true)\n\
900150983cd24fb0d6963f7d28e17f72\n\
b20a7076f5694be21ed71dfcd3164ff5\n\
900150983cd24fb0d6963f7d28e17f72|a9993e364706816aba3e25717850c26c9cd0d89d|ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad|16\n\
bool(true)\n\
900150983cd24fb0d6963f7d28e17f72\n\
3|900150983cd24fb0d6963f7d28e17f72\n\
0|d41d8cd98f00b204e9800998ecf8427e\n\
TypeError:hash_update(): Argument #1 ($context) must be a valid, non-finalized HashContext\n\
TypeError:hash_final(): Argument #1 ($context) must be a valid, non-finalized HashContext\n\
TypeError:hash_copy(): Argument #1 ($context) must be a valid, non-finalized HashContext\n\
ValueError:hash_file(): Argument #1 ($algo) must be a valid hashing algorithm\n\
Error:Call to private HashContext::__construct() from global scope\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(path);
}

#[test]
fn hash_tiger4_and_hash_context_object_boundaries_cover_public_rows() {
    let execution = run_source(
        r#"<?php
echo hash("tiger128,4", "I can't remember anything"), "\n";

$ctx = hash_init("tiger192,4");
hash_update($ctx, "I can't remember anything");
$copy = hash_copy($ctx);
echo hash_final($ctx), "|", hash_final($copy), "\n";

$clone_source = hash_init("tiger160,4");
hash_update($clone_source, "I can't remember anything");
$clone = clone $clone_source;
echo hash_final($clone_source), "|", hash_final($clone), "\n";

$ikm = "input key material";
echo bin2hex(hash_hkdf("tiger128,4", $ikm)), "\n";
echo bin2hex(hash_hkdf("tiger160,4", $ikm)), "\n";
echo bin2hex(hash_hkdf("tiger192,4", $ikm)), "\n";

ob_start();
var_dump(hash_init("sha256"));
$dump = ob_get_clean();
echo strpos($dump, "[\"algo\"]=>") !== false && strpos($dump, "sha256") !== false ? "debug\n" : "missing\n";

$finalized = hash_init("md5");
hash_final($finalized);
try {
    serialize($finalized);
} catch (Exception $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}

$hmac = hash_init("md5", HASH_HMAC, "key");
try {
    serialize($hmac);
} catch (Exception $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "a26ca3f58e74fb32ee44b099cb1b5122\n\
a26ca3f58e74fb32ee44b099cb1b512203375900f30b741d|a26ca3f58e74fb32ee44b099cb1b512203375900f30b741d\n\
a26ca3f58e74fb32ee44b099cb1b512203375900|a26ca3f58e74fb32ee44b099cb1b512203375900\n\
8acf517ecf58cccbd65c1186d71e4116\n\
cc0e33ee26700a2eb9a994bbb0e6cef29b429441\n\
97fa02d42331321fdc05c7f8dbc756d751ca36ce1aee69b0\n\
debug\n\
Exception:HashContext for algorithm \"md5\" cannot be serialized\n\
Exception:HashContext with HASH_HMAC option cannot be serialized\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_hmac_file_context_pbkdf2_and_hkdf_cover_bounded_rows() {
    let path = temp_hash_path("hmac");
    fs::write(
        &path,
        b"This is a sample string used to test the hash_hmac_file function with various hashing algorithms",
    )
    .unwrap();

    let source = format!(
        r#"<?php
echo function_exists("hash_hmac_file") && is_callable("hash_hmac_file") ? "file-fn" : "missing";
echo "|", function_exists("hash_hkdf") && is_callable("hash_hkdf") ? "hkdf-fn" : "missing", "\n";

$ctx = hash_init("md5", HASH_HMAC, str_repeat(chr(0x0b), 16));
hash_update($ctx, "Hi There");
echo hash_final($ctx), "\n";

$ctx = hash_init("md5", HASH_HMAC, "Jefe");
hash_update($ctx, "what do ya want for nothing?");
echo hash_final($ctx), "\n";

echo hash_hmac("md5", str_repeat(chr(0xDD), 50), str_repeat(chr(0xAA), 16)), "\n";
$content = "This is a sample string used to test the hash_hmac function with various hashing algorithms";
$key = "secret";
echo hash_hmac("md5", $content, $key), "\n";
echo bin2hex(hash_hmac("sha256", $content, $key, true)), "\n";

$file = {path:?};
echo hash_hmac_file("md5", $file, $key), "\n";
echo bin2hex(hash_hmac_file("sha256", $file, $key, true)), "\n";

foreach ([
    fn() => hash_hmac_file("foo", $file, $key, true),
    fn() => hash_hmac_file("crc32", $file, $key, true),
    fn() => hash_hmac_file("md5", $file . chr(0) . $file, $key, true),
] as $test) {{
    try {{
        var_dump($test());
    }} catch (\Error $e) {{
        echo get_class($e), ":", $e->getMessage(), "\n";
    }}
}}

echo hash_pbkdf2("sha1", "password", "salt", 1, 20), "\n";
echo hash_pbkdf2("sha1", "password", "salt", 1), "\n";
echo bin2hex(hash_pbkdf2("sha1", "password", "salt", 1, 20, true)), "\n";
echo hash_pbkdf2("sha256", "password", "salt", 1, 20), "\n";
echo bin2hex(hash_pbkdf2("sha256", "password", "salt", 1, 20, true)), "\n";
echo hash_pbkdf2("sha256", "passwordPASSWORDpassword", "saltSALTsaltSALTsaltSALTsaltSALTsalt", 4096, 40), "\n";

$ikm = "input key material";
echo bin2hex(hash_hkdf("md5", $ikm)), "\n";
echo bin2hex(hash_hkdf("Md5", $ikm, 7)), "\n";
echo bin2hex(hash_hkdf(
    "sha256",
    "\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b",
    42,
    "\xf0\xf1\xf2\xf3\xf4\xf5\xf6\xf7\xf8\xf9",
    "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c"
)), "\n";
echo bin2hex(hash_hkdf(
    "sha1",
    "\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b",
    42,
    "\xf0\xf1\xf2\xf3\xf4\xf5\xf6\xf7\xf8\xf9",
    "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c"
)), "\n";

foreach ([
    fn() => hash_hkdf("joaat", $ikm),
    fn() => hash_hkdf("sha1", ""),
    fn() => hash_hkdf("sha1", $ikm, -1),
    fn() => hash_hkdf("sha1", $ikm, 20 * 255 + 1),
] as $test) {{
    try {{
        var_dump($test());
    }} catch (\Error $e) {{
        echo get_class($e), ":", $e->getMessage(), "\n";
    }}
}}
"#
    );

    let execution = run_source(&source).unwrap();
    assert_eq!(
        execution.stdout,
        "file-fn|hkdf-fn\n\
9294727a3638bb1c13f48ef8158bfc9d\n\
750c783e6ab0b503eaa86e310a5db738\n\
56be34521d144c88dbb8c733f0e8b3f6\n\
2a632783e2812cf23de100d7d6a463ae\n\
49bde3496b9510a17d0edd8a4b0ac70148e32a1d51e881ec76faa96534125838\n\
8bddf39dd1c566c27acc7fa85ec36acf\n\
9135286ca4c84dec711e4b831f6cd39e672e5ff93d011321274eb76733cc1e40\n\
ValueError:hash_hmac_file(): Argument #1 ($algo) must be a valid cryptographic hashing algorithm\n\
ValueError:hash_hmac_file(): Argument #1 ($algo) must be a valid cryptographic hashing algorithm\n\
ValueError:hash_hmac_file(): Argument #2 ($filename) must not contain any null bytes\n\
0c60c80f961f0e71f3a9\n\
0c60c80f961f0e71f3a9b524af6012062fe037a6\n\
0c60c80f961f0e71f3a9b524af6012062fe037a6\n\
120fb6cffcf8b32c43e7\n\
120fb6cffcf8b32c43e7225256c4f837a86548c9\n\
348c89dbcbd32b2f32d814b8116e84cf2b17347e\n\
98b16391063ecee006a3ca8ee5776b1e\n\
98b16391063ece\n\
3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865\n\
085a01ea1b10f36933068b56efa5ad81a4f14b822f5b091568a9cdd4f155fda2c22e422478d305f3f896\n\
ValueError:hash_hkdf(): Argument #1 ($algo) must be a valid cryptographic hashing algorithm\n\
ValueError:hash_hkdf(): Argument #2 ($key) must not be empty\n\
ValueError:hash_hkdf(): Argument #3 ($length) must be greater than or equal to 0\n\
ValueError:hash_hkdf(): Argument #3 ($length) must be less than or equal to 5100\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);

    let _ = fs::remove_file(path);
}

#[test]
fn hash_pbkdf2_validates_algorithm_iterations_and_length_before_derivation_boundary() {
    let execution = run_source(
        r#"<?php
echo function_exists("hash_pbkdf2") ? "fn" : "missing";
echo "|", is_callable("hash_pbkdf2") ? "callable" : "missing";
$fn = new ReflectionFunction("hash_pbkdf2");
echo "|", $fn->getNumberOfRequiredParameters(), "/", $fn->getNumberOfParameters(), "\n";

foreach ([
    fn() => hash_pbkdf2("foo", "password", "salt", 1),
    fn() => hash_pbkdf2("crc32", "password", "salt", 1),
    fn() => hash_pbkdf2("md5", "password", "salt", 0),
    fn() => hash_pbkdf2("md5", "password", "salt", -1),
    fn() => hash_pbkdf2("md5", "password", "salt", 1, -1),
] as $test) {
    try {
        var_dump($test());
    } catch (\Error $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "fn|callable|4/6\n\
ValueError:hash_pbkdf2(): Argument #1 ($algo) must be a valid cryptographic hashing algorithm\n\
ValueError:hash_pbkdf2(): Argument #1 ($algo) must be a valid cryptographic hashing algorithm\n\
ValueError:hash_pbkdf2(): Argument #4 ($iterations) must be greater than 0\n\
ValueError:hash_pbkdf2(): Argument #4 ($iterations) must be greater than 0\n\
ValueError:hash_pbkdf2(): Argument #5 ($length) must be greater than or equal to 0\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);

    let tiger4 =
        run_source("<?php\necho hash_pbkdf2('tiger128,4', 'password', 'salt', 1), \"\\n\";\n")
            .unwrap();
    assert_eq!(tiger4.stdout, "8c9e1558d6e0476302660260ea0c6266\n");
}

#[test]
fn hash_equals_and_hmac_algorithm_metadata_cover_phpt_rows() {
    let execution = run_source(
        r#"<?php
echo hash_equals("same", "same") ? "same" : "bad";
echo "|", hash_equals("not1same", "not2same") ? "bad" : "diff";
echo "|", hash_equals("short", "longer") ? "bad" : "length";
echo "|", hash_equals("", "") ? "empty" : "bad", "\n";

$algos = hash_hmac_algos();
echo count($algos), "|", $algos[0], "|", $algos[5], "|", $algos[43], "|";
echo in_array("crc32", $algos, true) ? "bad" : "crypto-only";
echo "\n";

foreach ([
    fn() => hash_equals(123, "NaN"),
    fn() => hash_equals("NaN", 123),
    fn() => hash_equals(null, null),
] as $test) {
    try {
        var_dump($test());
    } catch (\Error $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "same|diff|length|empty\n\
44|md2|sha256|haval256,5|crypto-only\n\
TypeError:hash_equals(): Argument #1 ($known_string) must be of type string, int given\n\
TypeError:hash_equals(): Argument #2 ($user_string) must be of type string, int given\n\
TypeError:hash_equals(): Argument #1 ($known_string) must be of type string, null given\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_hash_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("hash") ? "1" : "0";
echo is_callable("hash_algos") ? "1" : "0";
echo function_exists("hash_init") ? "1" : "0";
echo is_callable("hash_init") ? "1" : "0";
echo function_exists("hash_update") ? "1" : "0";
echo is_callable("hash_update") ? "1" : "0";
echo function_exists("hash_final") ? "1" : "0";
echo is_callable("hash_final") ? "1" : "0";
echo function_exists("hash_copy") ? "1" : "0";
echo is_callable("hash_copy") ? "1" : "0";
echo function_exists("hash_file") ? "1" : "0";
echo is_callable("hash_file") ? "1" : "0";
echo function_exists("hash_hmac_file") ? "1" : "0";
echo is_callable("hash_hmac_file") ? "1" : "0";
echo function_exists("hash_update_file") ? "1" : "0";
echo is_callable("hash_update_file") ? "1" : "0";
echo function_exists("hash_update_stream") ? "1" : "0";
echo is_callable("hash_update_stream") ? "1" : "0";
echo function_exists("hash_hmac") ? "1" : "0";
echo is_callable("hash_hmac") ? "1" : "0";
echo function_exists("hash_hkdf") ? "1" : "0";
echo is_callable("hash_hkdf") ? "1" : "0";
echo function_exists("hash_pbkdf2") ? "1" : "0";
echo is_callable("hash_pbkdf2") ? "1" : "0";
echo function_exists("hash_hmac_algos") ? "1" : "0";
echo is_callable("hash_hmac_algos") ? "1" : "0";
echo function_exists("hash_equals") ? "1" : "0";
echo is_callable("hash_equals") ? "1" : "0";
echo function_exists("uniqid") ? "1" : "0";
echo is_callable("uniqid") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 30, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nhash('sha256', 'data');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\nhash_init('sha256');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\nhash_hmac('sha256', 'data', 'key');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error =
        emit_ir_source("<?php\nhash_hmac_file('sha256', 'file.txt', 'key');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\nhash_hkdf('sha256', 'key');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\nhash_pbkdf2('sha1', 'password', 'salt', 1);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\nhash_equals('same', 'same');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

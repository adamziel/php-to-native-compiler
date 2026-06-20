#!/usr/bin/env perl
use strict;
use warnings;

use Cwd qw(abs_path);
use FindBin qw($Bin);

my $repo_root = abs_path("$Bin/..");
my $php_src = $ENV{PHP_SRC_PHPT} // "/home/claude/php-src-phpt";

my $json_dir = "$php_src/ext/json";
my $header = "$json_dir/php_json.h";
my $arginfo = "$json_dir/json_arginfo.h";
my $json_c = "$json_dir/json.c";

for my $path ($header, $arginfo, $json_c) {
    die "missing php-src ext/json input: $path\n" unless -f $path;
}

my $revision = "unknown";
if (-d "$php_src/.git") {
    if (open my $git, "-|", "git", "-C", $php_src, "rev-parse", "HEAD") {
        my $line = <$git>;
        close $git;
        chomp $line if defined $line;
        $revision = $line if defined $line && $line ne "";
    }
}

my %value;
my %is_error;
my %allow_encode;
my %allow_decode;
my %allow_validate;
my @registered;

sub read_file {
    my ($path) = @_;
    open my $fh, "<", $path or die "read $path: $!\n";
    local $/;
    return <$fh>;
}

sub eval_c_int_expr {
    my ($expr) = @_;
    $expr =~ s/\s+//g;
    $expr =~ s{/\*.*?\*/}{}g;
    return int($1) if $expr =~ /\A(-?\d+)\z/;
    return 1 << int($1) if $expr =~ /\A\(?1<<(\d+)\)?\z/;
    die "unsupported JSON constant expression: $expr\n";
}

sub ptn_macro {
    my ($macro) = @_;
    $macro =~ s/\APHP_/PTN_/;
    return $macro;
}

sub c_quote_text {
    my ($text) = @_;
    $text =~ s/\\/\\\\/g;
    $text =~ s/"/\\"/g;
    return qq{"$text"};
}

sub rust_quote_text {
    my ($text) = @_;
    $text =~ s/\\/\\\\/g;
    $text =~ s/"/\\"/g;
    return qq{"$text"};
}

{
    open my $fh, "<", $header or die "read $header: $!\n";
    my $section = "";
    my $in_error_enum = 0;
    my $enum_next = 0;
    while (my $line = <$fh>) {
        if ($line =~ m{/\*\s*error codes\s*\*/}) {
            $in_error_enum = 1;
            next;
        }
        if ($in_error_enum) {
            if ($line =~ /^\s*}/) {
                $in_error_enum = 0;
                next;
            }
            if ($line =~ /^\s*(PHP_JSON_ERROR_[A-Z0-9_]+)(?:\s*=\s*(-?\d+))?\s*,/) {
                my ($macro, $explicit) = ($1, $2);
                my $constant_value = defined $explicit ? int($explicit) : $enum_next;
                $value{$macro} = $constant_value;
                $is_error{$macro} = 1;
                $enum_next = $constant_value + 1;
            }
            next;
        }

        if ($line =~ m{/\*\s*(.*?)\s*\*/}) {
            my $comment = $1;
            if ($comment eq "json_decode() options") {
                $section = "decode";
            } elsif ($comment eq "json_encode() options") {
                $section = "encode";
            } elsif ($comment eq "json_validate(), json_decode() and json_encode() common options") {
                $section = "all";
            } elsif ($comment eq "json_decode() and json_encode() common options") {
                $section = "decode_encode";
            } else {
                $section = "";
            }
        }

        if ($line =~ /^\s*#define\s+(PHP_JSON_[A-Z0-9_]+)\s+(.+?)\s*$/) {
            my ($macro, $expr) = ($1, $2);
            next if $macro eq "PHP_JSON_VERSION";
            next if $macro eq "PHP_JSON_PARSER_DEFAULT_DEPTH";
            next if $section eq "";
            my $constant_value = eval_c_int_expr($expr);
            $value{$macro} = $constant_value;
            if ($section eq "encode") {
                $allow_encode{$macro} = 1;
            } elsif ($section eq "decode") {
                $allow_decode{$macro} = 1;
            } elsif ($section eq "all") {
                $allow_encode{$macro} = 1;
                $allow_decode{$macro} = 1;
                $allow_validate{$macro} = 1;
            } elsif ($section eq "decode_encode") {
                $allow_encode{$macro} = 1;
                $allow_decode{$macro} = 1;
            }
        }
    }
}

{
    my $body = read_file($arginfo);
    while ($body =~ /REGISTER_LONG_CONSTANT\("([^"]+)",\s*(PHP_JSON_[A-Z0-9_]+),\s*CONST_PERSISTENT\);/g) {
        push @registered, { name => $1, macro => $2 };
    }
}

die "no JSON constants discovered in $arginfo\n" unless @registered;

my %error_message;
my $unknown_error_message;
my $location_format;
{
    open my $fh, "<", $json_c or die "read $json_c: $!\n";
    my $current_error;
    while (my $line = <$fh>) {
        if ($line =~ /case\s+(PHP_JSON_ERROR_[A-Z0-9_]+)\s*:/) {
            $current_error = $1;
            next;
        }
        if ($line =~ /default\s*:/) {
            $current_error = "__default__";
            next;
        }
        if (defined $current_error && $line =~ /return\s+"((?:[^"\\]|\\.)*)";/) {
            if ($current_error eq "__default__") {
                $unknown_error_message = $1;
            } else {
                $error_message{$current_error} = $1;
            }
            undef $current_error;
            next;
        }
        if ($line =~ /zend_strpprintf\s*\(\s*0\s*,\s*"((?:[^"\\]|\\.)*)"/) {
            $location_format = $1;
        }
    }
}

$unknown_error_message //= "Unknown error";
$location_format //= "%s near location %zu:%zu";

my %seen_name;
my @ordered;
for my $entry (@registered) {
    my ($name, $macro) = @{$entry}{qw(name macro)};
    die "duplicate JSON constant registration: $name\n" if $seen_name{$name}++;
    die "missing value for $macro from $header\n" unless exists $value{$macro};
    if ($is_error{$macro} && !exists $error_message{$macro}) {
        die "missing error message for $macro from $json_c\n";
    }
    push @ordered, $entry;
}

sub mask_expression {
    my ($kind) = @_;
    my @macros;
    for my $entry (@ordered) {
        my $macro = $entry->{macro};
        next if $is_error{$macro};
        my $allowed =
            $kind eq "encode" ? $allow_encode{$macro} :
            $kind eq "decode" ? $allow_decode{$macro} :
            $kind eq "validate" ? $allow_validate{$macro} :
            0;
        push @macros, ptn_macro($macro) if $allowed;
    }
    return @macros ? join(" | ", @macros) : "0";
}

sub flag_names_text {
    my ($kind) = @_;
    my @names;
    for my $entry (@ordered) {
        my $macro = $entry->{macro};
        next if $is_error{$macro};
        my $allowed =
            $kind eq "encode" ? $allow_encode{$macro} :
            $kind eq "decode" ? $allow_decode{$macro} :
            $kind eq "validate" ? $allow_validate{$macro} :
            0;
        push @names, $entry->{name} if $allowed;
    }
    return join(" | ", @names);
}

my $c_out = "$repo_root/src/backend/runtime/json_metadata.c";
open my $c, ">", $c_out or die "write $c_out: $!\n";
print $c "/* Generated by tools/generate-json-metadata.pl from php-src ext/json. */\n";
print $c "/* Source revision: $revision */\n";
print $c "/* Source files: ext/json/php_json.h, ext/json/json_arginfo.h, ext/json/json.c */\n\n";
for my $entry (@ordered) {
    my $macro = $entry->{macro};
    printf $c "#define %s %d\n", ptn_macro($macro), $value{$macro};
}
print $c "\n";
print $c "#define PTN_JSON_ENCODE_ALLOWED_FLAGS (" . mask_expression("encode") . ")\n";
print $c "#define PTN_JSON_DECODE_ALLOWED_FLAGS (" . mask_expression("decode") . ")\n";
print $c "#define PTN_JSON_VALIDATE_ALLOWED_FLAGS (" . mask_expression("validate") . ")\n";
print $c "#define PTN_JSON_VALIDATE_ALLOWED_FLAGS_TEXT " . c_quote_text(flag_names_text("validate")) . "\n";
print $c "#define PTN_JSON_ERROR_LOCATION_FORMAT " . c_quote_text($location_format) . "\n";
print $c "#define PTN_JSON_UNKNOWN_ERROR_MESSAGE " . c_quote_text($unknown_error_message) . "\n\n";
print $c "typedef struct {\n";
print $c "    const char *name;\n";
print $c "    int64_t value;\n";
print $c "    int is_error;\n";
print $c "    int encode_flag;\n";
print $c "    int decode_flag;\n";
print $c "    int validate_flag;\n";
print $c "    const char *message;\n";
print $c "} PtnJsonConstantMetadata;\n\n";
print $c "static const PtnJsonConstantMetadata ptn_json_constant_metadata_entries[] = {\n";
for my $entry (@ordered) {
    my ($name, $macro) = @{$entry}{qw(name macro)};
    my $message = $is_error{$macro} ? c_quote_text($error_message{$macro}) : "NULL";
    printf $c "    { %s, %s, %d, %d, %d, %d, %s },\n",
        c_quote_text($name),
        ptn_macro($macro),
        $is_error{$macro} ? 1 : 0,
        $allow_encode{$macro} ? 1 : 0,
        $allow_decode{$macro} ? 1 : 0,
        $allow_validate{$macro} ? 1 : 0,
        $message;
}
print $c "};\n\n";
print $c "static const size_t ptn_json_constant_metadata_len =\n";
print $c "    sizeof(ptn_json_constant_metadata_entries) / sizeof(ptn_json_constant_metadata_entries[0]);\n\n";
print $c "static PTN_UNUSED const PtnJsonConstantMetadata *ptn_json_constant_metadata_by_name(const char *name) {\n";
print $c "    for (size_t i = 0; i < ptn_json_constant_metadata_len; i++) {\n";
print $c "        if (strcmp(name, ptn_json_constant_metadata_entries[i].name) == 0) {\n";
print $c "            return &ptn_json_constant_metadata_entries[i];\n";
print $c "        }\n";
print $c "    }\n";
print $c "    return NULL;\n";
print $c "}\n\n";
print $c "static PTN_UNUSED const PtnJsonConstantMetadata *ptn_json_error_metadata_by_code(int error) {\n";
print $c "    for (size_t i = 0; i < ptn_json_constant_metadata_len; i++) {\n";
print $c "        const PtnJsonConstantMetadata *metadata = &ptn_json_constant_metadata_entries[i];\n";
print $c "        if (metadata->is_error && metadata->value == error) {\n";
print $c "            return metadata;\n";
print $c "        }\n";
print $c "    }\n";
print $c "    return NULL;\n";
print $c "}\n\n";
print $c "static PTN_UNUSED int ptn_json_constant_value(const char *name, int64_t *value_out) {\n";
print $c "    const PtnJsonConstantMetadata *metadata = ptn_json_constant_metadata_by_name(name);\n";
print $c "    if (metadata == NULL) {\n";
print $c "        return 0;\n";
print $c "    }\n";
print $c "    *value_out = metadata->value;\n";
print $c "    return 1;\n";
print $c "}\n\n";
print $c "static PTN_UNUSED void ptn_defined_constants_add_json(PtnValue table) {\n";
print $c "    for (size_t i = 0; i < ptn_json_constant_metadata_len; i++) {\n";
print $c "        const PtnJsonConstantMetadata *metadata = &ptn_json_constant_metadata_entries[i];\n";
print $c "        ptn_array_set_entry(table.as.array, ptn_array_string_key(metadata->name), ptn_int(metadata->value));\n";
print $c "    }\n";
print $c "}\n\n";
print $c "static PTN_UNUSED int ptn_reflection_constant_is_json(const char *name) {\n";
print $c "    return ptn_json_constant_metadata_by_name(name) != NULL;\n";
print $c "}\n\n";
print $c "static PTN_UNUSED const char *ptn_json_error_message(int error) {\n";
print $c "    const PtnJsonConstantMetadata *metadata = ptn_json_error_metadata_by_code(error);\n";
print $c "    return metadata == NULL ? PTN_JSON_UNKNOWN_ERROR_MESSAGE : metadata->message;\n";
print $c "}\n";
close $c;

my $rs_out = "$repo_root/src/json_metadata.rs";
open my $rs, ">", $rs_out or die "write $rs_out: $!\n";
print $rs "// Generated by tools/generate-json-metadata.pl from php-src ext/json.\n";
print $rs "// Source revision: $revision\n\n";
print $rs "pub(crate) const JSON_CONSTANT_NAMES: &[&str] = &[\n";
for my $entry (@ordered) {
    print $rs "    " . rust_quote_text($entry->{name}) . ",\n";
}
print $rs "];\n\n";
print $rs "pub(crate) fn is_json_constant_name(name: &str) -> bool {\n";
print $rs "    JSON_CONSTANT_NAMES.iter().any(|candidate| *candidate == name)\n";
print $rs "}\n";
close $rs;

print "generated $c_out\n";
print "generated $rs_out\n";

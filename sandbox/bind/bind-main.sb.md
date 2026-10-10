# Bind Sandbox

@@{ dataT: $f21 }
Amet magna do {minim} est $eu$ pariatur.

## JSON

{{{json[data-X]
{
  "string_and_unicode_escapes": {
    "basic_ascii": "The quick brown fox jumps over the lazy dog",
    "control_characters": "Line 1\nLine 2\r\n\tTabbed content \b \f",
    "quotes_and_slashes": "Escaped \"double quote\" and backslash: \\ /",
    "unicode_plane_0": "Indonesian: Selamat Pagi! \u00A9 \u00AE \u2122",
    "unicode_emojis_surrogates": "Rocket: \uD83D\uDE80, Fire: \uD83D\uDD25",
    "sanitization_test": "<script>alert(\"XSS & 'Quotes'\");</script>"
  },
  "numeric_precision_and_limits": {
    "zero_variants": [ 0, -0, 0.0, -0.0 ],
    "integers": [ 42, -99999, 2147483647, -2147483648 ],
    "safe_floating_points": [ 3.141592653589793, -0.000000123456789 ],
    "max_safe_integer_js": 9007199254740991,
    "min_safe_integer_js": -9007199254740991,
    "64bit_uint_overflow_test": 18446744073709551615,
    "scientific_notation": [ 1e5, 1E+5, 1e-5, 3.14e+2, -2.5E-3 ]
  },
  "structural_nesting_and_types": {
    "primitives": {
      "boolean_true": true,
      "boolean_false": false,
      "null_value": null,
      "empty_string": ""
    },
    "empty_containers": {
      "empty_object": {},
      "empty_array": []
    },
    "deeply_nested_array": [
      [
        [
          {
            "depth_level_4": "reached",
            "mixed_type_array": [ 100, "text", true, null, { "inner": "value" } ]
          }
        ]
      ]
    ]
  },
  "extreme_object_keys": {
    "": "empty string key",
    "12345": "numeric string key",
    "key with spaces and \t tabs": "whitespace key",
    "key.with.dot.notation": "dot key",
    "symbols_!@#$%^&*()_+-=[]{}|;:'\",.<>/?": "special symbols key",
    "unicode_key_\u0041\u0042\u0043": "ABC key"
  },
  "array_of_complex_records": [
    {
      "id": "usr_01",
      "is_active": true,
      "tags": [ "admin", "developer" ],
      "metrics": { "login_count": 142, "score": 98.6 }
    },
    {
      "id": "usr_02",
      "is_active": false,
      "tags": [],
      "metrics": { "login_count": 0, "score": null }
    }
  ]
}
}}}

@@{ dataX: $data-X }
Eiusmod qui sunt labore nisi.

## YAML

{{{yaml[foo_yi]
foo: bar

# 1. ANCHORS, ALIASES, & MERGE KEYS
default_config: &base_env
  timeout: 30
  retry: 3
  logging: &log_setting
    level: "DEBUG"
    destination: "/var/log/app.log"

development:
  <<: *base_env
  host: "localhost"
  port: 8080

production:
  <<: *base_env
  timeout: 10 # Override nilai dasar
  logging:
    <<: *log_setting
    level: "ERROR"

# 2. BLOCK SCALARS & MULTILINE STRINGS
text_formatting:
  folded_scalar: >
    Kalimat ini ditulis
    dalam beberapa baris,
    tetapi akan digabung
    menjadi satu baris tunggal.
  literal_scalar_strip: |-
    Baris 1
    Baris 2 (Garis baru dipertahankan, tanpa enter di akhir file)
  chomping_keep: |+
    Menjaga semua karakter newline di akhir skrip ini.

# 3. ADVANCED TYPES & EDGE CASES (RUST-COMPATIBLE)
complex_structures:
  # Explicit tags / casting
  string_as_int: !!str 12345
  explicit_null: !!null null
  binary_data: !!binary |
    R3VydSBTYW1wYWkgS3V0aXA=
  
  # Hex & Binary (otomatis di-parse ke angka)
  hex_and_bin: [ 0x1A, 0b1010 ]
  boolean_variations: [ true, false, "yes", "no" ]
  
  # String Key (Menggantikan Complex Key Array)
  "Latitude,Longitude": "Coord_HQ"

  # Multilevel inline JSON-like representation
  nested_flow_style: { array: [1, 2, { key: "value" }], status: active }
}}}


{{markerAx}}{yi-ro: $foo_yi}

Lorem {{markerDataX}}{data: $data-X, `marker-x`, --lenghth: "2rem", fooX: "roem"} ipsum.

## CSV

{{{csv[tabel21-karyawan4]
id,nama_pengguna,metadata_json,catatan_multiline,status_aktif
1,"Budi, S.T.",{"role":"admin","level":5},"Baris 1"
2,"Siti ""Blora""",{"tags":["dev","qa"]},"Baris 1
Baris 2 dengan ""kutip""",true
3,Andi,,NULL,false
4,Eka,"[""a"", ""b""]","Teks dengan koma, dan enter
di dalam string",TRUE
}}}

{{markerAx}}{yi-ro: $foo_yi}

Lorem {{markerDataX}}{data: $data-X, `marker-x`, --lenghth: "2rem", fooX: "roem"} ipsum.

===directiveCSV("Tabel Karyawan"){ tabel: { karyawan: $tabel21-karyawan4, foo: { fooYi: $foo_yi } }, --color: "#ffa"}

Incididunt laborum culpa Lorem proident nulla consequat dolor adipisicing.

{{markerTOML}}{yi-ro: $f21}

===

## TOML

{{{toml[f21]
# 1. SCALARS, SPECIAL FLOATS, & DATES
[primitives]
string_basic = "Hello\tWorld\nWith \"escapes\""
string_literal = 'C:\Users\System32\drivers\etc\hosts'
string_multiline_literal = '''
Baris 1
Baris 2 tanpa escape processing \n \t'''
integers = [ 42, +99, -17, 0xDEADBEEF, 0o755, 0b11010010, 1_000_000 ]
floats = [ +3.14159, -5e-6, 1.5e+10, inf, -inf, nan ]
booleans = [ true, false ]
datetimes = [ 1979-05-27T07:32:00Z, 1979-05-27T00:32:00-07:00, 1979-05-27, 07:32:00 ]

# 2. INLINE TABLES & MIXED/NESTED ARRAYS
[inline_structures]
inline_table = { name = "TOML", version = 1.0, features = ["fast", "clean"] }
nested_array = [ [ 1, 2 ], [ "a", "b" ], [ { key = "value" } ] ]
multiline_array = [
    "item_1",
    "item_2", # Trailing comma diizinkan
]

# 3. QUOTED KEYS & DOT-NOTATED KEYS
[advanced_keys]
"quoted.key.with.dots" = "valid"
"123_numeric_key" = 456
"key with spaces" = true
server.ip = "127.0.0.1" # Membuat sub-tabel { "server": { "ip": "127.0.0.1" } }

# 4. ARRAY OF TABLES (PARSED AS JSON ARRAY OF OBJECTS)
[[users]]
id = 1
name = "Budi"
roles = [ "admin", "dev" ]

[[users]]
id = 2
name = "Siti"
roles = [ "user" ]
}}}

{{markerTOML2}}@@("Lom Umami"){"Lom", lom: $f21}

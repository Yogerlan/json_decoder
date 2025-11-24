# JSON Decoder

A command-line utility for **decoding a proprietary, index-encoded JSON format** into fully expanded JSON.

This project was created as a **coding challenge**, where the client provided **only a sample input file** (with no description of the encoding rules), and the decoding algorithm was **deduced entirely by reverse-engineering** the provided example.

See the original sample input: [sample_in.txt](sample_in.txt)

------------------------------------------------------------------------

## 🧩 Background --- *A Reverse-Engineered Algorithm*

From this input, the decoder was designed to infer the rules:
- Arrays and objects contain values that are *not* the final content
- Many values are **indices** pointing into an encoded fragment list
- Pseudo-keys like `"_123"` represent *string keys* stored elsewhere
- Lines beginning with `"P<number>:"` define additional fragments
- Arrays like `["P", index]` represent pointer dereferences

------------------------------------------------------------------------

## ✨ Features

### Crates Used
- **`clap`** --- Command Line Argument Parsing
- **`serde_json`** --- JSON parsing, object manipulation, serialization, and pretty-printing
- **`regex`** --- Recognition and extraction of encoded indices such as `"_124"` or `"P326"`
- **`anyhow`** --- Rich contextual error handling

### Decoder Capabilities
- Index-based value decoding (supports positive & negative indices)
- Recursive decoding of arrays and objects
- Key indirection (`"_(\d+)"` → lookup string at index)
- Pointer array semantics (`["P", idx]`)
- Validation and error reporting for malformed input
- Pretty-formatted final output

------------------------------------------------------------------------

## 📦 Installation

```shell
git clone https://github.com/Yogerlan/json_decoder.git
cd json_decoder
cargo build --release
```

------------------------------------------------------------------------

## 🚀 Usage

```shell
json_decoder < sample_in.txt
```

### Input (reverse-engineered):
1. **First line**: a JSON array representing the base encoded fragment list
2. **Following lines**: pointer definitions of the form

        P<number>:<json_fragment>

3. **json_fragment**: these fragments collectively form a lookup table used during decoding

### Output:
Fully decoded pretty-formatted standard JSON

------------------------------------------------------------------------

## 🧠 Reverse-Engineered Decoding Algorithm

### 1. Build the encoded fragment list
- Parse the first line as a JSON array → `encoded_list`
- For each additional line `"P<number>:<json_fragment>"`, parse the fragment and place it at the specified index in `encoded_list`

### 2. Use the fragment at index `0` as the decoding root
```rust
decoded_data = decode_fragment(encoded_list[0])
```

### 3. Decoding primitives

#### Numbers
- Interpret positive numbers as direct indexes
- Negative numbers as offsets from the end
- Decode as `decode_fragment(encoded_list[index])`

#### Strings
- Normal strings remain unchanged
- Strings matching `^_(\d+)$` are indices for keys → lookup in `encoded_list` to obtain the real object key

#### Arrays
- Elements are decoded recursively
- Special form `["P", idx]` performs an immediate dereference

#### Objects
- Keys may be indirect (`"_(\d+)"`)
- Values decoded recursively

### 4. Output
Final data is pretty-printed with `serde_json::Serializer` + `PrettyFormatter`

------------------------------------------------------------------------

## 🧪 Testing --- *Coming soon*
```shell
cargo test
```

Suggested test areas:
-   Index resolution\
-   Negative indices\
-   Pointer arrays\
-   Key indirection\
-   Deep nesting\
-   Error handling

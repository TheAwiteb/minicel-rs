## Minicel-rs
A minimal excel-like formulas engine written in Rust without UI.

> Inspired by tsoding [Minicel](https://github.com/tsoding/minicel)

### Requirements
- Rust 1.65.0 or later (With Cargo)

### Installation
You can compile the project from source or download it with cargo (will compile it for you)

#### Compile from source
```bash
git clone https://github.com/theawiteb/minicel-rs.git
cd minicel-rs
cargo build --release
```
The binary will be in `target/release/minicel`

#### Download with cargo
```bash
cargo install --locked --git https://github.com/theawiteb/minicel-rs.git
```
The binary will be in `~/.cargo/bin/minicel`. Make sure to add `~/.cargo/bin` to your `PATH` to be able to run it from anywhere.

### Usage
```bash
minicel <input.csv> <output.csv>
```
The input file is the csv file that contains the formulas and the output file is the csv file that will contain the results.

### Syntax
The formulas are written in the cells/fields of the csv file. Is starts with `=` and then the **function call**
(The formula is only a function call)

#### Function call
> [!NOTE]
> Function name is case sensitive.

> [!NOTE]
> The function comes from the built-in functions only.

The function call is the name of the function followed by the arguments separated by `;` and surrounded by `(` and `)`.
Why `;`? Because `,` is used to separate the fields in the csv file.

#### Argument
The argument can be
| Type | Example |
| ---- | ------- |
| Number | `1`,`-1`,`0.5`,`-0.5` |
| String | `"Hello World"` |
| Field | `A1`,`B2`,`C3` |
| Function call | `sum(1;2)` |
| Bollean | `true`,`false` |
| Array | `[A1;2;sum(A2,A3)]` |

#### Array
An array is a list of values separated by `;` and surrounded by `[` and `]`.
The values can be any type of argument.

#### Operators
There is no operator precedence, you can use built-in functions to do the operations.

#### Built-in functions

> The functions are case sensitive, and the arguments are separated by `;`

|  Name  |                      Description                      | Number of arguments | Example    | Output |
| ------ |  ---------------------------------------------------  | ------------------- | ---------- | ------ |
| `print`| Prints the argument to the cell                       |          Any        | `print(A3)`|   38   | 
| `sum`  | Sums all the arguments                                |           2         | `sum(1;2)` |   3    |
| `sub`  | Subtracts the second argument from the first argument |           2         | `sub(1;2)` |  -1    |
| `mul`  | Multiplies all the arguments                          |           2         | `mul(2;3)` |   6    |
| `div`  | Divides the first argument by the second argument     |           2         | `div(6;2)` |   3    |

### Example
This is a simple example, for more examples see the [examples](examples) directory.
### Input
```csv
Name,Salary
John,1000
Jane,2000
Bob,=sum(B1;B2)
Dave,=sum(B3;mul(B2;0.8))
=print(A1),=print(B2)
```
The output will be
- `B3` will be `3000` because it's the sum of `B1` and `B2` wich is `1000` and `2000` respectively.
- `B4` will be `4600.0` because it's the sum of `B3` and `B2` multiplied by `0.8` wich is `3000` and `1600.0` respectively. (1600 is 80% of 2000)

> [!NOTE]
> The reason why `B4` is `4600.0` and not `4600` is because is a sum of a number and a float number and the result is a float number.

### Output
```csv
Name,Salary
John,1000
Jane,2000
Bob,3000
Dave,4600.0
John,2000
```

### Benchmarks
The following benchmarks were made on a 4.600GHz 12th Gen Intel i7-12700H CPU with 16GB of RAM. The input file is 100k rows of the above example.
```text
Benchmark 1: target/release/minicel test.csv out.csv
  Time (mean ± σ):      3.451 s ±  0.036 s    [User: 3.442 s, System: 0.009 s]
  Range (min … max):    3.406 s …  3.510 s    10 runs
```

Benchmarking was done with [hyperfine](https://github.com/sharkdp/hyperfine)

### License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details

### Contributing
Pull requests are welcome. Before making a pull request, please open an issue first to discuss what you would like to change.

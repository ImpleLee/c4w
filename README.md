# best 4w policy calculator

This program calculates the best 4w policy ("best" means maximize combo count).

## How to run

### Rotation System Builder
The program cannot calculate rotation system. You have to manually input the rotation system.

The `rs_builder` program can build the rotation system. It prints different problems to `stderr`, and you can answer whether the piece can be placed or not by pressing `y` or `n`.

The program will finally print the rotation system file in a binary format to its `stdout`. You should save it to a file.

```bash
$ cargo run --bin rs_builder > rs.bin
```

The `rs_builder` program can only find fields beginning from a certain field. The field currently can only be specified by manually editing the `rs_builder.rs` file (at line `queue.push_back(Field([...]));`)

### Main Program
The main program can calculate the best 4w policy given the rotation system, preview count, whether it can hold the pieces or not, and piece sequence pattern (current only supports random).

These parameters except the rotation system can be specified by manually editing the `main.rs` file (at line `let num2state = RandomStates::new(&continuations, 6, false);`).

The program reads the rotation system file from its `stdin` and prints the best 4w policy to its `stdout`. Some runtime information is printed to `stderr`.

```bash
$ cargo run --bin c4w < rs.bin > ren-count.txt
```

The program prints the average maximum combo count for each piece (averaged over all possible piece sequences). You can customize the print result by adding instances of `Printer` trait in the `printer/mod.rs` file.

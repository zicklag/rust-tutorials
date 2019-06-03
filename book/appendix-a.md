# Appendix A: Rust Library Types

## Library Types Evaluation

Here is my current understanding of different Rust library types.

### Rlib

`rlib`'s are Rust static libraries. They contain all of the metadata and code necessary to build a Rust crate and link it into another Rust crate statically. Given just the `rlib` for a crate, you can include that crate into a Rust program by using `extern crate crate_name`. Even if you are dynamically linking a crate like a `.dll`, `.so`, etc., you will still need to have the `rlib` to include that crate into another crate because they shared library is missing some of the required metadata ( I think ).

### Cdylib

`cdylib`'s are primarily designed for building shared libraries that can be linked into C/C++ programs.

- They have a minimum size of ~2.2M on Linux, so the smallest `cdylib` is larger than the smallest `dylib`. I don't have any idea why the minimum size is so large.
- They are designed for building a C ABI that can be dynamically linked to C/C++ programs ( and Rust programs that define extern blocks )
  - If you want to expose any portion of the a `cdylib`'s interface over the C ABI you must use **Extern Functions**.
  - If you want to link a Rust program to a Rust `cdylib` over the C ABI you have to use **Extern Blocks** ( See Rust reference doc ).
- When building a `cdylib`, any functions that are **not** exposed through an extern block will be automatically stripped from the generated library as an optimization.
  - For example, if you build a `cdylib` for a large rust library and you do not export any functions using the extern keyword, the library will be essentially empty and will be about 2.2M ( because that is the minimum size for a `cdylib` ).

### Dylib

`dylib`'s are Rust shared libraries which have an unstable ABI that can change across Rust releases.

- To build a `dylib`, you probably need the `-C prefer-dynamic` rustflag set so that it will not attempt to statically link the standard library.
- Within the same Rust version, you can dynamically link to a Rust `dylib` from a Rust crate **without** having to use extern functions or blocks because it will go over the Rust ABI.
- `dylib`'s will **not** strip out any unused functions when built. Even if none of the functions in the library are explicitly expose or used in the crate, all of the functions will still be included in the generated library.

## Proposal for Creating a Stable Rust Dynamic Linking Strategy

> **Note:** This is just a draft for my thoughts about a way to make dynamic linking more stable and usable in Rust.

The largest problem with dynamically linking Rust libraries, as far as I understand it, is that there is no stable ABI for Rust right now and, according to some, there may never be. Still, dynamic linking can be very useful, like in my use-case of creating a Rust plugin system, where plugins have access to the application's full Rust interface.

> My proposal is to provide a way to tell the Rust compiler to automatically expose all public Rust interfaces in a crate over the C ABI and also provide a way to import the external crate over the C ABI into another Rust crate.

The problem with using `dylib` for creating Rust dynamic libraries is that you cannot link to them from Rust crates that are not built using the **exact** same version of the Rust compiler. The problem with using `cdylib` is that it only exposes the items that are explicitly exposed using the `extern` keyword. While this makes sense for building C/C++ interfaces, it does not make sense for trying to link a full Rust library to another rust library as that would require you to manually create extern functions and blocks for the entire Rust interface you want to link to.

If there was a way to build a Rust library with a C ABI and import it over that ABI, the library could be compatible with different rust versions without having to stabilize Rust's own ABI. I'm imagining that this would have something like the following changes:

- There is now a new `rcdylib` ( Rust C Dylib, probably a better name for that ) crate type that will cause Rust to build a shared library that exposes all of the Rust functions over the C ABI.
- There is now a new way to import external crates: `extern "C" crate crate_name;` Including a crate into another crate like this will cause it to use the C ABI to call any functions on that imported crate.

### Caveats

#### Derive Macros

In order to facilitate dynamically linking to Rust libraries that provide derive macros, it might be required that derive plugins be compiled as `rcdylib`'s as well. Otherwise, if you are attempting to compile Rust crate A and dynamically link it to Rust crate B, and the version of Rust you are using to compile crate A is different than crate B, the rustc binary compiling crate A will not be able to successfully link to the derive macro shared libraries that were built using crate B's version of Rust.

### Disclaimer

I don't fully understand the way that Rust libraries are linked and I don't have any in-depth knowledge of the compiler. This proposal is made based on the best information that I could obtain right now. I would like to get feedback on whether or not what I'm saying makes any sense, and whether or not this could be feasible to work into Rust.

If there is another way to achieve what I am trying to achieve, then I would be happy to go with an alternative that doesn't require changing the Rust compiler.

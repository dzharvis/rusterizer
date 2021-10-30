# Rusterizer
This project implements a basic OpenGL rendering pipeline. 
No dependencies were used, writtent entirely in Rust from scratch.
You can try it online here [https://dzharvis.github.io/rusterizer/](https://dzharvis.github.io/rusterizer/)

<img src="img/1.png" width="600"/>
<p float="left">
  <img src="img/3.png" width="300"/>
  <img src="img/2.png" width="300"/>
</p>

## Build
### Locally
#### Prerequisites
 - rustup
 - cargo

```bash
> rustup update
> rustup default nightly
> cargo run --features=local
```

### Web
#### Prerequisites
 - trunk
 - python3
 - rustup
 - cargo

```bash
> rustup update
> rustup default nightly
> rustup target add wasm32-unknown-unknown
> ./build.sh
```

### Kudos
This project was implemented by following the [ssloy/tinyrenderer](https://github.com/ssloy/tinyrenderer) lessons.

### Todo
 - [ ] Write manual SIMD for matrices
 - [ ] Refactor code

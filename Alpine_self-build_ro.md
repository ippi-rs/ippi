# Para compilar ippi.rs en Alpine y correrlo como sistema read-only (como PiKVM) el approach correcto es:

```

# En Alpine instala el toolchain Rust musl
apk add rust cargo musl-dev

# Cross-compile para Pi Zero W (arm)
cargo build --target arm-unknown-linux-musleabihf --release

# Para Pi Zero 2W / Orange Pi (aarch64)
cargo build --target aarch64-unknown-linux-musl --release

```

#### El resultado es un binario estático musl 

— cero dependencias, corre en Alpine read-only sin drama, tratando de usarlo sin glibc, sin gcompat.
— Se produces el binario correcto desde el inicio.

##### dependencias claves para Alpine

 - glibc gcompact  -> apk add gcompat libstdc++

# Se crea el archivo
touch personas.db

# Instalamos target musl
rustup target add x86_64-unknown-linux-musl

# Instalamos musl-tools
sudo apt update
sudo apt install musl-tools

# Para compilar modo producción
cargo build --release --target x86_64-unknown-linux-musl

# Para ejecutar
./target/x86_64-unknown-linux-musl/release/personas_crud

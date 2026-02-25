# Create file
touch persons.db

# Install target musl
rustup target add x86_64-unknown-linux-musl

# Install musl-tools
sudo apt update
sudo apt install musl-tools

# Compile production environment
cargo build --release --target x86_64-unknown-linux-musl

# Execute
./target/x86_64-unknown-linux-musl/release/persons_crud

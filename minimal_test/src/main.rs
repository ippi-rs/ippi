use thiserror::Error;

#[derive(Error, Debug)]
enum TestError {
    #[error("Test error")]
    Test,
}

fn main() -> Result<(), TestError> {
    println!("=== Minimal Rust Test ===");
    println!("✅ Rust compila correctamente");
    println!("✅ Dependencies work");
    Ok(())
}
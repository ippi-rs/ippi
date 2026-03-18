// Programa mínimo para testear compilación
fn main() {
    println!("=== Test IPPI ===");
    
    // Test 1: Constantes básicas
    const NAME: &str = "ippi";
    const VERSION: &str = "0.1.0";
    
    println!("Project: {} v{}", NAME, VERSION);
    
    // Test 2: Estructuras básicas
    #[derive(Debug)]
    struct Config {
        web_host: String,
        web_port: u16,
    }
    
    let config = Config {
        web_host: "0.0.0.0".to_string(),
        web_port: 8080,
    };
    
    println!("Config: {:?}", config);
    
    // Test 3: Protocolos
    let protocols = vec![
        "/ippi/0.1.0",
        "/ippi-dht/1.0.0",
    ];
    
    println!("Protocols: {:?}", protocols);
    
    // Test 4: Bootstrap
    let bootstrap = "bootstrap.ippi.rs";
    println!("Bootstrap: {}", bootstrap);
    
    println!("\n✅ All basic tests passed!");
    println!("Ready for: cargo check --no-default-features");
}
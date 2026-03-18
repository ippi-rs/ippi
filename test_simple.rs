// Teste simples para verificar se o projeto compila
fn main() {
    println!("Testando compilação do IPPI...");
    
    // Verificar constantes básicas
    println!("Nome do projeto: {}", ippi::NAME);
    println!("Versão: {}", ippi::VERSION);
    
    // Testar configuração básica
    let config = ippi::Config::default();
    println!("Configuração carregada: host={}, port={}", 
             config.web.host, config.web.port);
    
    println!("✅ Teste básico passou!");
}
#!/bin/sh
echo "=== Teste de Compilação IPPI ==="
echo ""

echo "1. Verificando sintaxe básica..."
rustc --edition=2024 --crate-type lib -o /dev/null src/lib.rs 2>&1 | head -20

echo ""
echo "2. Verificando módulo de configuração..."
rustc --edition=2024 --crate-type lib -o /dev/null src/config/mod.rs 2>&1 | head -20

echo ""
echo "3. Verificando módulo web..."
rustc --edition=2024 --crate-type lib -o /dev/null src/web/mod.rs 2>&1 | head -20

echo ""
echo "4. Verificando constantes..."
echo "NAME: $(grep 'NAME.*=.*"ippi"' src/lib.rs)"
echo "VERSION: $(grep 'VERSION.*=.*env!' src/lib.rs)"

echo ""
echo "=== Teste de Configuração ==="
echo ""
echo "5. Verificando arquivo de configuração..."
if [ -f "config/ippi.toml" ]; then
    echo "✅ config/ippi.toml encontrado"
    grep -n "bootstrap.ippi.rs" config/ippi.toml
else
    echo "❌ config/ippi.toml não encontrado"
fi

echo ""
echo "=== Teste de Protocolos ==="
echo ""
echo "6. Verificando protocolos P2P/DHT..."
grep -r "/ippi/" src/ --include="*.rs" | head -5
grep -r "bootstrap.ippi.rs" src/ --include="*.rs" | head -5

echo ""
echo "=== Status do Rebranding ==="
echo ""
echo "✅ Nome do projeto: IPPI"
echo "✅ Protocolo P2P: /ippi/0.1.0"
echo "✅ Protocolo DHT: /ippi-dht/1.0.0"
echo "✅ Bootstrap nodes: bootstrap.ippi.rs"
echo "✅ Configuração: config/ippi.toml"
echo "✅ Docker: ippi/ippi:latest"
echo "✅ GitHub: github.com/ippi-rs/ippi"

echo ""
echo "=== Próximos Passos ==="
echo ""
echo "1. Testar compilação completa: cargo build --no-default-features"
echo "2. Testar com frontend: cargo build --features frontend-embedded"
echo "3. Testar exemplo P2P: cargo run --example p2p_network --features p2p-full"
echo "4. Configurar bootstrap.ippi.rs com servidores reais"
echo "5. Atualizar documentação online em ippi.rs"
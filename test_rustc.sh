#!/bin/sh
echo "=== Probando compilación con rustc ==="

# Probar compilación del archivo lib.rs
echo "1. Probando lib.rs..."
rustc --edition=2024 --crate-type lib src/lib.rs --out-dir /tmp/ippi_test 2>&1

# Probar main.rs (necesita dependencias)
echo ""
echo "2. Probando sintaxis de main.rs..."
rustc --edition=2024 --crate-type bin src/main.rs --out-dir /tmp/ippi_test 2>&1 | head -20

# Verificar constantes
echo ""
echo "3. Verificando constantes..."
grep -n "NAME.*=.*\"ippi\"" src/lib.rs
grep -n "VERSION.*=.*env!" src/lib.rs

# Verificar protocolos
echo ""
echo "4. Verificando protocolos..."
grep -r "/ippi/" src/ --include="*.rs" | grep -v "test_" | head -5

echo ""
echo "=== Estado de compilación ==="
if [ -f "/tmp/ippi_test/lib.rlib" ]; then
    echo "✅ lib.rs compila correctamente"
else
    echo "⚠️  Hay errores en lib.rs"
fi

echo ""
echo "=== Próximo paso recomendado ==="
echo "Ejecutar: cargo check --no-default-features"
echo "Si hay errores, corregirlos antes de continuar"
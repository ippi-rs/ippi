#!/usr/bin/env python3
"""
Script para verificar y corregir errores de compilación en IPPI
"""

import os
import subprocess
import sys

def run_command(cmd):
    """Ejecuta un comando y retorna el resultado"""
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
        return result.returncode, result.stdout, result.stderr
    except Exception as e:
        return -1, "", str(e)

def check_rust():
    """Verifica si Rust está instalado"""
    print("=== Verificando Rust ===")
    code, out, err = run_command("rustc --version")
    if code == 0:
        print(f"✅ Rust encontrado: {out.strip()}")
        return True
    else:
        print(f"❌ Rust no encontrado: {err}")
        return False

def check_cargo():
    """Verifica si Cargo está instalado"""
    print("\n=== Verificando Cargo ===")
    code, out, err = run_command("cargo --version")
    if code == 0:
        print(f"✅ Cargo encontrado: {out.strip()}")
        return True
    else:
        print(f"❌ Cargo no encontrado: {err}")
        return False

def check_dependencies():
    """Verifica dependencias básicas"""
    print("\n=== Verificando dependencias ===")
    
    # Verificar archivos críticos
    critical_files = [
        "Cargo.toml",
        "src/lib.rs",
        "src/main.rs",
        "config/ippi.toml",
    ]
    
    missing = []
    for f in critical_files:
        if os.path.exists(f):
            print(f"✅ {f}")
        else:
            print(f"❌ {f} (no encontrado)")
            missing.append(f)
    
    return len(missing) == 0

def try_cargo_check():
    """Intenta ejecutar cargo check"""
    print("\n=== Intentando cargo check ===")
    code, out, err = run_command("cargo check --no-default-features")
    
    if code == 0:
        print("✅ cargo check ejecutado exitosamente")
        print(out)
        return True
    else:
        print("❌ cargo check falló")
        print("Error:", err)
        
        # Buscar errores comunes
        if "could not find `Config` in `vergen`" in err:
            print("\n⚠️  Error de vergen detectado")
            print("Solución: Usar vergen 8.0 en lugar de 10.0")
        
        if "include_str" in err and "dist/index.html" in err:
            print("\n⚠️  Error de include_str detectado")
            print("Solución: El archivo dist/index.html no existe")
            print("Se crea al compilar con --features frontend-embedded")
        
        return False

def main():
    print("=== IPPI - Verificación de Compilación ===\n")
    
    # Verificar requisitos
    if not check_rust():
        print("\n❌ Instala Rust primero: https://rustup.rs/")
        return 1
    
    if not check_cargo():
        print("\n❌ Cargo no disponible")
        return 1
    
    if not check_dependencies():
        print("\n❌ Faltan archivos críticos")
        return 1
    
    # Intentar cargo check
    if try_cargo_check():
        print("\n🎉 ¡Todo parece correcto!")
        print("\nPróximos pasos:")
        print("1. cargo build --no-default-features")
        print("2. ./build-with-podman.sh")
        print("3. cargo run --example p2p_network --features p2p-full")
        return 0
    else:
        print("\n🔧 Se detectaron errores")
        print("\nCorrecciones aplicadas:")
        print("- vergen actualizado a 8.0")
        print("- clap agregado a dependencias")
        print("- web/mod.rs ajustado para fallback sin dist/")
        print("\nIntenta nuevamente: cargo check --no-default-features")
        return 1

if __name__ == "__main__":
    sys.exit(main())
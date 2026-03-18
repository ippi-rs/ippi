use std::fs;
use std::path::Path;
use std::process::Command;

fn generate_build_info() -> Result<(), Box<dyn std::error::Error>> {
    vergen_gitcl::Emitter::default().emit()?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate build information
    generate_build_info()?;
    
    // Build frontend if feature is enabled
    if cfg!(feature = "frontend-embedded") {
        build_frontend()?;
    }
    
    // Print rerun-if-changed directives
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");
    
    // Watch frontend directory if feature enabled
    if cfg!(feature = "frontend-embedded") {
        println!("cargo:rerun-if-changed=frontend/");
    }
    
    Ok(())
}


fn build_frontend() -> Result<(), Box<dyn std::error::Error>> {
    let frontend_dir = Path::new("frontend");
    
    // Check if frontend directory exists
    if !frontend_dir.exists() {
        println!("cargo:warning=Frontend directory not found, creating placeholder");
        create_placeholder_frontend()?;
        return Ok(());
    }
    
    // Check if package.json exists
    let package_json = frontend_dir.join("package.json");
    if !package_json.exists() {
        println!("cargo:warning=package.json not found in frontend directory");
        create_placeholder_frontend()?;
        return Ok(());
    }
    
    // Check if node_modules exists
    let node_modules = frontend_dir.join("node_modules");
    if !node_modules.exists() {
        println!("cargo:warning=node_modules not found, attempting to install dependencies");
        
        // Try to install dependencies
        let status = Command::new("npm")
            .arg("install")
            .current_dir(frontend_dir)
            .status();
            
        match status {
            Ok(status) if status.success() => {
                println!("cargo:warning=Frontend dependencies installed successfully");
            }
            _ => {
                println!("cargo:warning=Failed to install frontend dependencies, creating placeholder");
                create_placeholder_frontend()?;
                return Ok(());
            }
        }
    }
    
    // Build frontend
    println!("cargo:warning=Building frontend...");
    
    let status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(frontend_dir)
        .status();
    
    match status {
        Ok(status) if status.success() => {
            println!("cargo:warning=Frontend built successfully");
            
            // Verify dist directory was created
            let dist_dir = Path::new("dist");
            if !dist_dir.exists() {
                println!("cargo:warning=dist directory not created by build, creating placeholder");
                create_placeholder_frontend()?;
            }
            
            // Check for index.html
            let index_html = dist_dir.join("index.html");
            if !index_html.exists() {
                println!("cargo:warning=index.html not found in dist, creating placeholder");
                create_placeholder_frontend()?;
            }
        }
        Ok(status) => {
            println!("cargo:warning=Frontend build failed with exit code: {:?}", status.code());
            create_placeholder_frontend()?;
        }
        Err(e) => {
            println!("cargo:warning=Failed to run frontend build: {}", e);
            create_placeholder_frontend()?;
        }
    }
    
    Ok(())
}

fn create_placeholder_frontend() -> Result<(), Box<dyn std::error::Error>> {
    let dist_dir = Path::new("dist");
    
    // Create dist directory if it doesn't exist
    if !dist_dir.exists() {
        fs::create_dir_all(dist_dir)?;
    }
    
    // Create minimal index.html
    let index_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>IPPI - Lightweight P2P KVM-over-IP</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            color: white;
        }
        
        .container {
            text-align: center;
            max-width: 600px;
            padding: 2rem;
            background: rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(10px);
            border-radius: 20px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
        }
        
        .logo {
            font-size: 3rem;
            margin-bottom: 1rem;
        }
        
        h1 {
            font-size: 2.5rem;
            margin-bottom: 1rem;
            background: linear-gradient(45deg, #fff, #f0f0f0);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        
        .status {
            display: inline-block;
            padding: 0.5rem 1rem;
            background: rgba(76, 175, 80, 0.2);
            border: 1px solid rgba(76, 175, 80, 0.5);
            border-radius: 20px;
            margin: 1rem 0;
            font-weight: bold;
        }
        
        .message {
            margin: 1.5rem 0;
            line-height: 1.6;
            opacity: 0.9;
        }
        
        .features {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin: 2rem 0;
        }
        
        .feature {
            background: rgba(255, 255, 255, 0.05);
            padding: 1rem;
            border-radius: 10px;
            transition: transform 0.2s;
        }
        
        .feature:hover {
            transform: translateY(-2px);
        }
        
        .feature-icon {
            font-size: 2rem;
            margin-bottom: 0.5rem;
        }
        
        .warning {
            background: rgba(255, 193, 7, 0.2);
            border: 1px solid rgba(255, 193, 7, 0.5);
            border-radius: 10px;
            padding: 1rem;
            margin: 1rem 0;
        }
        
        .links {
            margin-top: 2rem;
            display: flex;
            gap: 1rem;
            justify-content: center;
            flex-wrap: wrap;
        }
        
        .btn {
            padding: 0.75rem 1.5rem;
            background: rgba(255, 255, 255, 0.1);
            border: 1px solid rgba(255, 255, 255, 0.2);
            border-radius: 10px;
            color: white;
            text-decoration: none;
            transition: all 0.2s;
        }
        
        .btn:hover {
            background: rgba(255, 255, 255, 0.2);
            transform: translateY(-1px);
        }
        
        .btn-primary {
            background: rgba(59, 130, 246, 0.5);
            border-color: rgba(59, 130, 246, 0.8);
        }
        
        .btn-primary:hover {
            background: rgba(59, 130, 246, 0.7);
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="logo">🚀</div>
        <h1>IPPI</h1>
        <div class="status">Backend Running</div>
        
        <div class="message">
            Lightweight P2P KVM-over-IP for Raspberry Pi
        </div>
        
        <div class="warning">
            ⚠️ Frontend is in development mode. Full interface coming soon!
        </div>
        
        <div class="features">
            <div class="feature">
                <div class="feature-icon">🔌</div>
                <h3>P2P Networking</h3>
                <p>Auto-discovery via Kademlia DHT</p>
            </div>
            
            <div class="feature">
                <div class="feature-icon">🌐</div>
                <h3>NAT Traversal</h3>
                <p>Works behind NAT without port forwarding</p>
            </div>
            
            <div class="feature">
                <div class="feature-icon">🎥</div>
                <h3>WebRTC Video</h3>
                <p>Low-latency video streaming</p>
            </div>
            
            <div class="feature">
                <div class="feature-icon">☁️</div>
                <h3>Cloud-init</h3>
                <p>Auto-provisioning VMs</p>
            </div>
        </div>
        
        <div class="links">
            <a href="/api/health" class="btn">Health Check</a>
            <a href="/api/docs" class="btn">API Docs</a>
            <a href="https://github.com/ippi-rs/ippi" class="btn btn-primary" target="_blank">GitHub</a>
        </div>
        
        <div style="margin-top: 2rem; font-size: 0.9rem; opacity: 0.7;">
            Built with Rust 🦀 • Running on Raspberry Pi
        </div>
    </div>
    
    <script>
        // Basic JavaScript for dynamic updates
        document.addEventListener('DOMContentLoaded', function() {
            // Update status periodically
            function updateStatus() {
                fetch('/api/health')
                    .then(response => response.json())
                    .then(data => {
                        const statusEl = document.querySelector('.status');
                        if (data.status === 'healthy') {
                            statusEl.textContent = 'Backend Running ✓';
                            statusEl.style.background = 'rgba(76, 175, 80, 0.2)';
                            statusEl.style.borderColor = 'rgba(76, 175, 80, 0.5)';
                        } else {
                            statusEl.textContent = 'Backend Issues ⚠️';
                            statusEl.style.background = 'rgba(255, 193, 7, 0.2)';
                            statusEl.style.borderColor = 'rgba(255, 193, 7, 0.5)';
                        }
                    })
                    .catch(() => {
                        const statusEl = document.querySelector('.status');
                        statusEl.textContent = 'Connection Error ❌';
                        statusEl.style.background = 'rgba(244, 67, 54, 0.2)';
                        statusEl.style.borderColor = 'rgba(244, 67, 54, 0.5)';
                    });
            }
            
            // Update status every 10 seconds
            updateStatus();
            setInterval(updateStatus, 10000);
            
            // Add click animation to buttons
            document.querySelectorAll('.btn').forEach(btn => {
                btn.addEventListener('click', function(e) {
                    this.style.transform = 'scale(0.95)';
                    setTimeout(() => {
                        this.style.transform = '';
                    }, 200);
                });
            });
        });
    </script>
</body>
</html>"#;
    
    fs::write(dist_dir.join("index.html"), index_html)?;
    
    // Create minimal CSS file
    let css_dir = dist_dir.join("assets");
    fs::create_dir_all(&css_dir)?;
    
    let style_css = r#"/* Minimal styles for IPPI placeholder */
body {
    margin: 0;
    padding: 0;
    font-family: sans-serif;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
}

.header {
    text-align: center;
    margin-bottom: 40px;
}

.features {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 20px;
}

.feature-card {
    border: 1px solid #ddd;
    border-radius: 8px;
    padding: 20px;
    text-align: center;
}

.feature-icon {
    font-size: 2em;
    margin-bottom: 10px;
}"#;
    
    fs::write(css_dir.join("style.css"), style_css)?;
    
    println!("cargo:warning=Created placeholder frontend in dist/");
    
    Ok(())
}

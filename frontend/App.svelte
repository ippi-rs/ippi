<script>
  let config = $state({
    version: '0.1.0',
    web: { host: '0.0.0.0', port: 8080 },
    p2p: { enabled: false },
    webrtc: { enabled: false },
    cloud_init: { enabled: false }
  });
  
  let uptime = $state('0s');
  let peers = $state(0);
  let memory = $state('0 MB');
  let logs = $state([
    '[System] IPPI v0.1.0 starting...',
    '[Web] Server listening on 0.0.0.0:8080',
    '[API] Health endpoint available at /health'
  ]);
  
  let startTime = $state(Date.now());
  
  async function loadConfig() {
    try {
      const response = await fetch('/api/config');
      config = await response.json();
    } catch (error) {
      console.error('Failed to load config:', error);
      addLog(`[Error] Failed to load config: ${error.message}`);
    }
  }
  
  async function updatePeers() {
    try {
      const response = await fetch('/api/p2p/peers');
      const data = await response.json();
      peers = data.count || 0;
    } catch (error) {
      peers = 0;
    }
  }
  
  function updateUptime() {
    const elapsed = Math.floor((Date.now() - startTime) / 1000);
    const hours = Math.floor(elapsed / 3600);
    const minutes = Math.floor((elapsed % 3600) / 60);
    const seconds = elapsed % 60;
    
    let uptimeStr = '';
    if (hours > 0) uptimeStr += `${hours}h `;
    if (minutes > 0 || hours > 0) uptimeStr += `${minutes}m `;
    uptimeStr += `${seconds}s`;
    
    uptime = uptimeStr;
  }
  
  function updateMemory() {
    if (performance.memory) {
      const usedMB = Math.round(performance.memory.usedJSHeapSize / 1024 / 1024);
      memory = `${usedMB} MB`;
    }
  }
  
  function addLog(message) {
    logs = [...logs.slice(-9), message];
  }
  
  function openConsole() {
    addLog('[Console] Launching KVM console...');
    alert('Console feature coming soon!');
  }
  
  function openSettings() {
    addLog('[Settings] Opening configuration panel...');
    alert('Settings panel coming soon!');
  }
  
  $effect(() => {
    loadConfig();
    updateUptime();
    updatePeers();
    updateMemory();
    
    const uptimeInterval = setInterval(updateUptime, 1000);
    const peersInterval = setInterval(updatePeers, 5000);
    const memoryInterval = setInterval(updateMemory, 3000);
    
    addLog('[System] Frontend initialized successfully');
    
    return () => {
      clearInterval(uptimeInterval);
      clearInterval(peersInterval);
      clearInterval(memoryInterval);
    };
  });
</script>

<div class="container">
  <header class="header">
    <div class="logo">
      <i class="fas fa-server"></i>
    </div>
    <h1>IPPI</h1>
    <p class="tagline">Lightweight P2P KVM-over-IP for Raspberry Pi Zero</p>
    <div class="status-badge {config.p2p?.enabled ? 'status-online' : 'status-offline'}">
      {config.p2p?.enabled ? 'Connected' : 'Offline'}
    </div>
  </header>
  
  <div class="stats">
    <div class="stat">
      <div class="stat-value">{uptime}</div>
      <div class="stat-label">Uptime</div>
    </div>
    <div class="stat">
      <div class="stat-value">v{config.version}</div>
      <div class="stat-label">Version</div>
    </div>
    <div class="stat">
      <div class="stat-value">{peers}</div>
      <div class="stat-label">P2P Peers</div>
    </div>
    <div class="stat">
      <div class="stat-value">{memory}</div>
      <div class="stat-label">Memory Used</div>
    </div>
  </div>
  
  <div class="grid">
    <div class="card">
      <div class="card-header">
        <i class="fas fa-tv card-icon"></i>
        <h3>KVM Console</h3>
      </div>
      <p>Access virtual machine consoles remotely with low-latency video streaming.</p>
      <button class="button" on:click={openConsole}>
        <i class="fas fa-play"></i> Launch Console
      </button>
    </div>
    
    <div class="card">
      <div class="card-header">
        <i class="fas fa-network-wired card-icon"></i>
        <h3>P2P Network</h3>
      </div>
      <p>Zero-configuration networking with automatic peer discovery behind NAT.</p>
      <div>
        <span class="status-badge {config.p2p?.enabled ? 'status-online' : 'status-offline'}">
          {config.p2p?.enabled ? 'Enabled' : 'Disabled'}
        </span>
      </div>
    </div>
    
    <div class="card">
      <div class="card-header">
        <i class="fas fa-cloud card-icon"></i>
        <h3>Cloud-Init</h3>
      </div>
      <p>Automated VM provisioning with cloud-init metadata and userdata support.</p>
      <div>
        <span class="status-badge {config.cloud_init?.enabled ? 'status-online' : 'status-offline'}">
          {config.cloud_init?.enabled ? 'Enabled' : 'Disabled'}
        </span>
      </div>
    </div>
    
    <div class="card">
      <div class="card-header">
        <i class="fas fa-cogs card-icon"></i>
        <h3>Configuration</h3>
      </div>
      <p>Manage system settings, network configuration, and service preferences.</p>
      <button class="button button-secondary" on:click={openSettings}>
        <i class="fas fa-sliders-h"></i> Settings
      </button>
    </div>
  </div>
  
  <div class="card">
    <div class="card-header">
      <i class="fas fa-terminal card-icon"></i>
      <h3>System Logs</h3>
    </div>
    <div class="logs">
      {#each logs as log}
        <div>{log}</div>
      {/each}
    </div>
  </div>
  
  <footer class="footer">
    <p>IPPI &copy; 2025 | Lightweight P2P KVM-over-IP | Built with Rust & Svelte</p>
    <p>
      <a href="https://github.com/ippi-rs/ippi" style="color: var(--primary); text-decoration: none;">
        <i class="fab fa-github"></i> GitHub
      </a>
      &nbsp;|&nbsp;
      <a href="/docs" style="color: var(--primary); text-decoration: none;">
        <i class="fas fa-book"></i> Documentation
      </a>
    </p>
  </footer>
</div>

<style>
  .container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem 1rem;
  }
  
  .header {
    text-align: center;
    margin-bottom: 3rem;
    padding: 2rem;
    background: rgba(30, 41, 59, 0.7);
    border-radius: 1rem;
    border: 1px solid rgba(148, 163, 184, 0.2);
  }
  
  .logo {
    font-size: 3rem;
    color: #3b82f6;
    margin-bottom: 1rem;
  }
  
  .tagline {
    font-size: 1.2rem;
    color: #94a3b8;
    margin-bottom: 2rem;
  }
  
  .status-badge {
    display: inline-block;
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.875rem;
    font-weight: 600;
  }
  
  .status-online {
    background: rgba(16, 185, 129, 0.2);
    color: #10b981;
  }
  
  .status-offline {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }
  
  .stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin: 2rem 0;
  }
  
  .stat {
    text-align: center;
    padding: 1rem;
    background: rgba(30, 41, 59, 0.7);
    border-radius: 0.75rem;
  }
  
  .stat-value {
    font-size: 2rem;
    font-weight: 700;
    color: #3b82f6;
    margin-bottom: 0.5rem;
  }
  
  .stat-label {
    font-size: 0.875rem;
    color: #94a3b8;
  }
  
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1.5rem;
    margin-bottom: 2rem;
  }
  
  .card {
    background: rgba(30, 41, 59, 0.7);
    border: 1px solid rgba(148, 163, 184, 0.2);
    border-radius: 1rem;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
    transition: transform 0.3s, border-color 0.3s;
  }
  
  .card:hover {
    transform: translateY(-2px);
    border-color: #3b82f6;
  }
  
  .card-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }
  
  .card-icon {
    font-size: 1.5rem;
    color: #3b82f6;
  }
  
  .button {
    background: #3b82f6;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 0.5rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.3s;
    margin-top: 1rem;
  }
  
  .button:hover {
    background: #2563eb;
  }
  
  .button-secondary {
    background: rgba(148, 163, 184, 0.2);
    color: #f8fafc;
  }
  
  .button-secondary:hover {
    background: rgba(148, 163, 184, 0.3);
  }
  
  .logs {
    background: rgba(15, 23, 42, 0.7);
    border-radius: 0.5rem;
    padding: 1rem;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 0.875rem;
    max-height: 200px;
    overflow-y: auto;
    white-space: pre-wrap;
    color: #94a3b8;
  }
  
  .footer {
    text-align: center;
    margin-top: 3rem;
    padding-top: 2rem;
    border-top: 1px solid rgba(148, 163, 184, 0.2);
    color: #94a3b8;
  }
  
  body {
    background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
    color: #f8fafc;
    min-height: 100vh;
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
  }
</style>
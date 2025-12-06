import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';

// State
let deviceInfo = null;
let peers = [];
let connectedPeer = null;
let transfers = [];
let isScanning = false;

// DOM Elements
const app = document.getElementById('app');

// Initialize app
async function init() {
  render();
  
  // Get device info
  deviceInfo = await invoke('get_device_info');
  
  // Start discovery
  await invoke('start_discovery');
  isScanning = true;
  
  // Set up event listeners
  await setupEventListeners();
  
  // Poll for peers
  setInterval(async () => {
    peers = await invoke('get_peers');
    updatePeersList();
  }, 1000);
  
  // Poll for transfers
  setInterval(async () => {
    transfers = await invoke('get_transfers');
    updateTransfersList();
  }, 500);
  
  updateUI();
}

async function setupEventListeners() {
  await listen('peers-updated', async () => {
    peers = await invoke('get_peers');
    updatePeersList();
  });
  
  await listen('peer-connected', (event) => {
    connectedPeer = event.payload;
    updateUI();
  });
  
  await listen('peer-disconnected', () => {
    connectedPeer = null;
    updateUI();
  });
  
  await listen('transfer-updated', async () => {
    transfers = await invoke('get_transfers');
    updateTransfersList();
  });
  
  await listen('transfer-progress', (event) => {
    const [transferId, bytes] = event.payload;
    updateTransferProgress(transferId, bytes);
  });
  
  await listen('incoming-transfer', (event) => {
    const [transferId, files] = event.payload;
    showIncomingTransfer(transferId, files);
  });
}

function render() {
  app.innerHTML = `
    <header class="header">
      <div class="logo">
        <img src="/logo.svg" alt="Thunderlink" class="logo-icon" />
        <span class="logo-text">Thunderlink</span>
      </div>
      <div class="device-info">
        <div class="device-badge">
          <span class="platform-icon">${getPlatformIcon()}</span>
          <span id="device-name">Loading...</span>
        </div>
      </div>
    </header>
    
    <main class="main-content">
      <aside class="sidebar">
        <div class="sidebar-header">
          <div class="sidebar-title">Nearby Devices</div>
          <div class="search-peers">
            <span>🔍</span>
            <input type="text" placeholder="Search devices..." id="peer-search">
          </div>
        </div>
        <div class="peers-list" id="peers-list">
          <div class="no-peers">
            <div class="no-peers-icon">📡</div>
            <div>Searching for devices...</div>
            <div style="font-size: 12px; margin-top: 8px;">Connect via Thunderbolt or USB-C</div>
          </div>
        </div>
        <div class="scanning-indicator" id="scanning-indicator">
          <span class="scanning-dot"></span>
          <span>Scanning network...</span>
        </div>
      </aside>
      
      <section class="transfer-area">
        <div class="transfer-header">
          <div class="connection-status">
            <div class="connection-indicator disconnected" id="connection-indicator">
              <span class="dot"></span>
              <span id="connection-text">Not Connected</span>
            </div>
            <div class="connected-peer-info" id="connected-peer-info" style="display: none;">
              <span id="connected-peer-name"></span>
            </div>
          </div>
          <div class="transfer-actions">
            <button class="btn btn-secondary" id="btn-settings" title="Settings">
              ⚙️ Settings
            </button>
            <button class="btn btn-primary" id="btn-send" disabled>
              📤 Send Files
            </button>
          </div>
        </div>
        
        <div class="drop-zone-container">
          <div class="drop-zone disabled" id="drop-zone">
            <div class="drop-zone-content">
              <div class="drop-zone-icon">📁</div>
              <div class="drop-zone-title">Drop files here to send</div>
              <div class="drop-zone-subtitle">Or click to browse • Connect to a device first</div>
            </div>
          </div>
          
          <div class="transfers-section" id="transfers-section" style="display: none;">
            <div class="transfers-header">
              <span class="transfers-title">Transfers</span>
              <button class="btn btn-secondary" style="padding: 6px 12px; font-size: 12px;" id="btn-clear-transfers">
                Clear Completed
              </button>
            </div>
            <div class="transfers-list" id="transfers-list"></div>
          </div>
        </div>
      </section>
    </main>
  `;
  
  // Setup event handlers
  setupDragDrop();
  setupButtons();
}

function setupDragDrop() {
  const dropZone = document.getElementById('drop-zone');
  
  dropZone.addEventListener('dragover', (e) => {
    e.preventDefault();
    if (connectedPeer) {
      dropZone.classList.add('drag-over');
    }
  });
  
  dropZone.addEventListener('dragleave', () => {
    dropZone.classList.remove('drag-over');
  });
  
  dropZone.addEventListener('drop', async (e) => {
    e.preventDefault();
    dropZone.classList.remove('drag-over');
    
    if (!connectedPeer) return;
    
    const files = Array.from(e.dataTransfer.files);
    if (files.length > 0) {
      await sendFiles(files.map(f => f.path));
    }
  });
  
  dropZone.addEventListener('click', async () => {
    if (!connectedPeer) return;
    await selectAndSendFiles();
  });
}

function setupButtons() {
  document.getElementById('btn-send').addEventListener('click', selectAndSendFiles);
  
  document.getElementById('btn-settings').addEventListener('click', () => {
    // TODO: Open settings modal
    console.log('Settings clicked');
  });
  
  document.getElementById('btn-clear-transfers').addEventListener('click', () => {
    transfers = transfers.filter(t => 
      t.status !== 'Completed' && t.status !== 'Failed' && t.status !== 'Cancelled'
    );
    updateTransfersList();
  });
  
  document.getElementById('peer-search').addEventListener('input', (e) => {
    const query = e.target.value.toLowerCase();
    const filteredPeers = peers.filter(p => 
      p.name.toLowerCase().includes(query) || 
      p.platform.toLowerCase().includes(query)
    );
    renderPeersList(filteredPeers);
  });
}

async function selectAndSendFiles() {
  if (!connectedPeer) return;
  
  const selected = await open({
    multiple: true,
    title: 'Select files to send',
  });
  
  if (selected) {
    const paths = Array.isArray(selected) ? selected : [selected];
    await sendFiles(paths);
  }
}

async function sendFiles(filePaths) {
  try {
    await invoke('send_files', { filePaths });
  } catch (error) {
    console.error('Failed to send files:', error);
    alert('Failed to send files: ' + error);
  }
}

async function connectToPeer(peerId) {
  try {
    await invoke('connect_to_peer', { peerId });
    connectedPeer = peers.find(p => p.id === peerId);
    updateUI();
  } catch (error) {
    console.error('Failed to connect:', error);
    alert('Failed to connect: ' + error);
  }
}

async function disconnectPeer() {
  try {
    await invoke('disconnect_peer');
    connectedPeer = null;
    updateUI();
  } catch (error) {
    console.error('Failed to disconnect:', error);
  }
}

function updateUI() {
  // Update device name
  if (deviceInfo) {
    document.getElementById('device-name').textContent = deviceInfo.name;
  }
  
  // Update connection status
  const indicator = document.getElementById('connection-indicator');
  const connText = document.getElementById('connection-text');
  const peerInfo = document.getElementById('connected-peer-info');
  const dropZone = document.getElementById('drop-zone');
  const sendBtn = document.getElementById('btn-send');
  
  if (connectedPeer) {
    indicator.classList.remove('disconnected');
    indicator.classList.add('connected');
    connText.textContent = 'Connected';
    peerInfo.style.display = 'flex';
    document.getElementById('connected-peer-name').textContent = connectedPeer.name;
    dropZone.classList.remove('disabled');
    sendBtn.disabled = false;
    
    // Update drop zone text
    dropZone.querySelector('.drop-zone-subtitle').textContent = 'Or click to browse files';
  } else {
    indicator.classList.remove('connected');
    indicator.classList.add('disconnected');
    connText.textContent = 'Not Connected';
    peerInfo.style.display = 'none';
    dropZone.classList.add('disabled');
    sendBtn.disabled = true;
    
    // Update drop zone text
    dropZone.querySelector('.drop-zone-subtitle').textContent = 'Or click to browse • Connect to a device first';
  }
  
  updatePeersList();
}

function updatePeersList() {
  renderPeersList(peers);
}

function renderPeersList(peerList) {
  const container = document.getElementById('peers-list');
  
  if (peerList.length === 0) {
    container.innerHTML = `
      <div class="no-peers">
        <div class="no-peers-icon">📡</div>
        <div>Searching for devices...</div>
        <div style="font-size: 12px; margin-top: 8px;">Connect via Thunderbolt or USB-C</div>
      </div>
    `;
    return;
  }
  
  container.innerHTML = peerList.map(peer => `
    <div class="peer-card ${connectedPeer?.id === peer.id ? 'connected' : ''}" 
         data-peer-id="${peer.id}">
      <div class="peer-avatar">${peer.name.charAt(0).toUpperCase()}</div>
      <div class="peer-info">
        <div class="peer-name">${escapeHtml(peer.name)}</div>
        <div class="peer-meta">
          <span class="peer-platform">${getPlatformIconForPeer(peer.platform)} ${peer.platform}</span>
          <span>•</span>
          <span>${peer.ip}</span>
        </div>
      </div>
      <div class="peer-status"></div>
    </div>
  `).join('');
  
  // Add click handlers
  container.querySelectorAll('.peer-card').forEach(card => {
    card.addEventListener('click', () => {
      const peerId = card.dataset.peerId;
      if (connectedPeer?.id === peerId) {
        disconnectPeer();
      } else {
        connectToPeer(peerId);
      }
    });
  });
}

function updateTransfersList() {
  const section = document.getElementById('transfers-section');
  const container = document.getElementById('transfers-list');
  
  if (transfers.length === 0) {
    section.style.display = 'none';
    return;
  }
  
  section.style.display = 'block';
  
  container.innerHTML = transfers.map(transfer => `
    <div class="transfer-item" data-transfer-id="${transfer.id}">
      <div class="transfer-icon">${transfer.is_sending ? '📤' : '📥'}</div>
      <div class="transfer-details">
        <div class="transfer-name">${escapeHtml(transfer.file_name)}</div>
        <div class="transfer-progress">
          <div class="transfer-progress-bar" style="width: ${getTransferProgress(transfer)}%"></div>
        </div>
        <div class="transfer-meta">
          <span class="transfer-status ${transfer.status.toLowerCase()}">
            ${getStatusIcon(transfer.status)} ${transfer.status}
          </span>
          <span class="transfer-speed">${formatSpeed(transfer.speed_bps)}</span>
          <span>${formatSize(transfer.transferred)} / ${formatSize(transfer.file_size)}</span>
        </div>
      </div>
      ${transfer.status === 'Transferring' ? `
        <button class="transfer-cancel" data-transfer-id="${transfer.id}">✕</button>
      ` : ''}
    </div>
  `).join('');
  
  // Add cancel handlers
  container.querySelectorAll('.transfer-cancel').forEach(btn => {
    btn.addEventListener('click', async (e) => {
      e.stopPropagation();
      const transferId = btn.dataset.transferId;
      await invoke('cancel_transfer', { transferId });
    });
  });
}

function updateTransferProgress(transferId, bytes) {
  const item = document.querySelector(`[data-transfer-id="${transferId}"]`);
  if (!item) return;
  
  const transfer = transfers.find(t => t.id === transferId);
  if (!transfer) return;
  
  const progressBar = item.querySelector('.transfer-progress-bar');
  if (progressBar) {
    const percent = transfer.file_size > 0 ? (bytes / transfer.file_size) * 100 : 0;
    progressBar.style.width = `${percent}%`;
  }
}

function showIncomingTransfer(transferId, files) {
  // For now, just log it. Could show a notification
  console.log('Incoming transfer:', transferId, files);
}

// Utility functions
function getTransferProgress(transfer) {
  if (transfer.file_size === 0) return 100;
  return (transfer.transferred / transfer.file_size) * 100;
}

function getStatusIcon(status) {
  switch (status) {
    case 'Completed': return '✓';
    case 'Failed': return '✕';
    case 'Cancelled': return '⊘';
    case 'Transferring': return '↔';
    case 'Connecting': return '◐';
    default: return '○';
  }
}

function formatSpeed(bps) {
  if (!bps || bps === 0) return '';
  const KB = 1024;
  const MB = KB * 1024;
  const GB = MB * 1024;
  
  if (bps >= GB) return `${(bps / GB).toFixed(2)} GB/s`;
  if (bps >= MB) return `${(bps / MB).toFixed(2)} MB/s`;
  if (bps >= KB) return `${(bps / KB).toFixed(2)} KB/s`;
  return `${bps} B/s`;
}

function formatSize(bytes) {
  if (!bytes) return '0 B';
  const KB = 1024;
  const MB = KB * 1024;
  const GB = MB * 1024;
  
  if (bytes >= GB) return `${(bytes / GB).toFixed(2)} GB`;
  if (bytes >= MB) return `${(bytes / MB).toFixed(2)} MB`;
  if (bytes >= KB) return `${(bytes / KB).toFixed(2)} KB`;
  return `${bytes} B`;
}

function getPlatformIcon() {
  const platform = navigator.platform.toLowerCase();
  if (platform.includes('mac')) return '🍎';
  if (platform.includes('win')) return '🪟';
  if (platform.includes('linux')) return '🐧';
  return '💻';
}

function getPlatformIconForPeer(platform) {
  switch (platform) {
    case 'macOS': return '🍎';
    case 'Windows': return '🪟';
    case 'Linux': return '🐧';
    default: return '💻';
  }
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Start the app
init();


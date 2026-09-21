// RusTTY Webview  SPA Application (Vanilla JS)
// Complete client-side application: routing, state, IPC, views, components

'use strict';


// ─── Lucide SVG Icon Paths (embedded, no CDN) ────────────────────────────────
const ICON_PATHS = {
  home: '<path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/><path d="M3 10a2 2 0 0 1 .709-1.528l7-5.999a2 2 0 0 1 2.582 0l7 5.999A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
  sparkles: '<path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/><path d="M5 3v4"/><path d="M19 17v4"/><path d="M3 5h4"/><path d="M17 19h4"/>',
  rocket: '<path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/><path d="m12 15-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/><path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/>',
  server: '<rect width="20" height="8" x="2" y="2" rx="2" ry="2"/><rect width="20" height="8" x="2" y="14" rx="2" ry="2"/><line x1="6" x2="6.01" y1="6" y2="6"/><line x1="6" x2="6.01" y1="18" y2="18"/>',
  'server-plus': '<rect width="20" height="8" x="2" y="2" rx="2" ry="2"/><rect width="20" height="8" x="2" y="14" rx="2" ry="2"/><line x1="6" x2="6.01" y1="6" y2="6"/><line x1="6" x2="6.01" y1="18" y2="18"/><path d="M16 6h4"/><path d="M18 4v4"/>',
  network: '<rect x="16" y="16" width="6" height="6" rx="1"/><rect x="2" y="16" width="6" height="6" rx="1"/><rect x="9" y="2" width="6" height="6" rx="1"/><path d="M5 16v-3a1 1 0 0 1 1-1h12a1 1 0 0 1 1 1v3"/><path d="M12 12V8"/>',
  settings: '<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/>',
  paintbrush: '<path d="M18.37 2.63 14 7l-1.59-1.59a2 2 0 0 0-2.82 0L8 7l9 9 1.59-1.59a2 2 0 0 0 0-2.82L17 10l4.37-4.37a2.12 2.12 0 1 0-3-3Z"/><path d="M9 8c-2 3-4 3.5-7 4l8 10c2-1 6-5 6-7"/><path d="M14.5 17.5 4.5 15"/>',
  'book-open': '<path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>',
  terminal: '<polyline points="4 17 10 11 4 5"/><line x1="12" x2="20" y1="19" y2="19"/>',
  plug: '<path d="M12 22v-5"/><path d="M9 8V2"/><path d="M15 8V2"/><path d="M18 8v5a6 6 0 0 1-6 6 6 6 0 0 1-6-6V8Z"/>',
  user: '<path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>',
  monitor: '<rect width="20" height="14" x="2" y="3" rx="2"/><line x1="8" x2="16" y1="21" y2="21"/><line x1="12" x2="12" y1="17" y2="21"/>',
  lock: '<rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>',
  eye: '<path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/>',
  'eye-off': '<path d="M9.88 9.88a3 3 0 1 0 4.24 4.24"/><path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"/><path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"/><line x1="2" x2="22" y1="2" y2="22"/>',
  shield: '<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/>',
  'shield-alert': '<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/><path d="M12 8v4"/><path d="M12 16h.01"/>',
  save: '<path d="M15.2 3a2 2 0 0 1 1.4.6l3.8 3.8a2 2 0 0 1 .6 1.4V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z"/><path d="M17 21v-7a1 1 0 0 0-1-1H8a1 1 0 0 0-1 1v7"/><path d="M7 3v4a1 1 0 0 0 1 1h7"/>',
  x: '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>',
  undo: '<path d="M3 7v6h6"/><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"/>',
  trash: '<path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/>',
  edit: '<path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/><path d="m15 5 4 4"/>',
  'alert-triangle': '<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/><path d="M12 9v4"/><path d="M12 17h.01"/>',
  check: '<path d="M20 6 9 17l-5-5"/>',
  'globe-lock': '<path d="M15.686 15A14.5 14.5 0 0 1 12 22a14.5 14.5 0 0 1 0-20 10 10 0 1 0 9.542 13"/><path d="M2 12h8.5"/><path d="M20 6V4a2 2 0 1 0-4 0v2"/><rect width="8" height="5" x="14" y="6" rx="1"/>',
  folder: '<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>',
  'folder-plus': '<path d="M12 10v6"/><path d="M9 13h6"/><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>',
  'folder-symlink': '<path d="m2 13 3-3 3 3"/><path d="M5 10v7a2 2 0 0 0 2 2h12"/><path d="M22 13V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>',
  'heart-plus': '<path d="M13.5 2.764a4.97 4.97 0 0 0-2.83 1.3l-.67.66-.67-.66a5 5 0 0 0-7.08 7.07L12 20.84l3.7-3.71"/><path d="M16 14v6"/><path d="M19 17h-6"/>',
  'at-sign': '<circle cx="12" cy="12" r="4"/><path d="M16 8v5a3 3 0 0 0 6 0v-1a10 10 0 1 0-4 8"/>',
  globe: '<circle cx="12" cy="12" r="10"/><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"/><path d="M2 12h20"/>',
  plus: '<path d="M5 12h14"/><path d="M12 5v14"/>',
  'chevron-right': '<path d="m9 18 6-6-6-6"/>',
  'chevron-down': '<path d="m6 9 6 6 6-6"/>',
  move: '<polyline points="5 9 2 12 5 15"/><polyline points="9 5 12 2 15 5"/><polyline points="15 19 12 22 9 19"/><polyline points="19 9 22 12 19 15"/><line x1="2" x2="22" y1="12" y2="12"/><line x1="12" x2="12" y1="2" y2="22"/>',
  search: '<circle cx="11" cy="11" r="8"/><path d="m21 21-4.34-4.34"/>',
  'folder-search': '<path d="M10.7 20H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H20a2 2 0 0 1 2 2v4.1"/><path d="m21 21-1.9-1.9"/><circle cx="17" cy="17" r="3"/>',
  'book-search': '<path d="M11 22H5.5a1 1 0 0 1 0-5h4.501"/><path d="m21 22-1.879-1.878"/><path d="M3 19.5v-15A2.5 2.5 0 0 1 5.5 2H18a1 1 0 0 1 1 1v8"/><circle cx="17" cy="18" r="3"/>',
  info: '<circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>',
  'arrow-up-right': '<path d="M7 7h10v10"/><path d="M7 17 17 7"/>',
  'arrow-right': '<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>',
  'grip-vertical': '<circle cx="9" cy="12" r="1"/><circle cx="9" cy="5" r="1"/><circle cx="9" cy="19" r="1"/><circle cx="15" cy="12" r="1"/><circle cx="15" cy="5" r="1"/><circle cx="15" cy="19" r="1"/>',
  cpu: '<path d="M12 20v2"/><path d="M12 2v2"/><path d="M17 20v2"/><path d="M17 2v2"/><path d="M2 12h2"/><path d="M2 17h2"/><path d="M2 7h2"/><path d="M20 12h2"/><path d="M20 17h2"/><path d="M20 7h2"/><path d="M7 20v2"/><path d="M7 2v2"/><rect x="4" y="4" width="16" height="16" rx="2"/><rect x="8" y="8" width="8" height="8" rx="1"/>',
  gpu: '<path d="M2 17h18a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2H2"/><path d="M2 21V3"/><path d="M7 17v3a1 1 0 0 0 1 1h5a1 1 0 0 0 1-1v-3"/><circle cx="16" cy="11" r="2"/><circle cx="8" cy="11" r="2"/>',
  'hard-drive': '<path d="M10 16h.01"/><path d="M2.212 11.577a2 2 0 0 0-.212.896V18a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-5.527a2 2 0 0 0-.212-.896L18.55 5.11A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/><path d="M21.946 12.013H2.054"/><path d="M6 16h.01"/>',
  'memory-stick': '<path d="M12 12v-2"/><path d="M12 18v-2"/><path d="M16 12v-2"/><path d="M16 18v-2"/><path d="M2 11h1.5"/><path d="M20 18v-2"/><path d="M20.5 11H22"/><path d="M4 18v-2"/><path d="M8 12v-2"/><path d="M8 18v-2"/><rect x="2" y="6" width="20" height="10" rx="2"/>',
};

const HOST_AVAILABLE_ICONS = [
  { id: 'terminal', label: 'Terminal' },
  { id: 'server', label: 'Servidor' },
  { id: 'server-plus', label: 'Cluster' },
  { id: 'monitor', label: 'Estação' },
  { id: 'cpu', label: 'CPU' },
  { id: 'gpu', label: 'GPU / IA' },
  { id: 'hard-drive', label: 'Storage' },
  { id: 'memory-stick', label: 'Memória' },
  { id: 'network', label: 'Rede' },
  { id: 'globe', label: 'Nuvem' },
  { id: 'globe-lock', label: 'Gateway' },
  { id: 'shield', label: 'Firewall' },
  { id: 'lock', label: 'Bastion' },
  { id: 'folder', label: 'Arquivos' },
  { id: 'folder-search', label: 'Documentação' },
  { id: 'book-search', label: 'Manual' },
  { id: 'plug', label: 'Dispositivo' },
  { id: 'rocket', label: 'Produção' },
  { id: 'save', label: 'Backup' },
  { id: 'user', label: 'Usuário' },
];

const FOLDER_COLORS = [
  { name: 'Laranja RusTTY', value: '#FF7300' },
  { name: 'Azul Elétrico', value: '#3B82F6' },
  { name: 'Verde Esmeralda', value: '#10B981' },
  { name: 'Roxo Neon', value: '#8B5CF6' },
  { name: 'Vermelho Coral', value: '#EF4444' },
  { name: 'Âmbar Dourado', value: '#F59E0B' },
  { name: 'Ciano Vibrante', value: '#06B6D4' },
  { name: 'Rosa Magenta', value: '#EC4899' },
];

const FOLDER_AVAILABLE_ICONS = [
  'folder', 'folder-search', 'book-search', 'server', 'server-plus', 'network',
  'globe', 'globe-lock', 'shield', 'lock', 'cpu', 'gpu', 'hard-drive', 'save', 'rocket'
];

function getHostIconLabel(id) {
  const item = HOST_AVAILABLE_ICONS.find(i => i.id === id);
  return item ? item.label : (id || 'Terminal');
}

/** Creates an SVG icon element */
function icon(name, size = 20) {
  const paths = ICON_PATHS[name];
  if (!paths) return '';
  return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${paths}</svg>`;
}

//  IPC Layer 

let liveWs = null;

function connectWebSocket() {
  const urlParams = new URLSearchParams(window.location.search);
  const port = window.WS_PORT || urlParams.get('ws_port');
  if (!port) {
    console.log('[RusTTY] WS_PORT not found, relying on native IPC fallback');
    return;
  }
  try {
    const ws = new WebSocket(`ws://127.0.0.1:${port}`);
    liveWs = ws;

    ws.onopen = () => {
      console.log('[RusTTY] Live WebSocket connected on port', port);
      IPC.requestConfig();
      IPC.requestClientConfig();
    };

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        handleRustMessage(msg);
      } catch (err) {
        console.error('[RusTTY] WS JSON parse error:', err);
      }
    };

    ws.onclose = () => {
      console.log('[RusTTY] Live WebSocket disconnected, retrying in 1s...');
      liveWs = null;
      setTimeout(connectWebSocket, 1000);
    };

    ws.onerror = (err) => {
      console.warn('[RusTTY] WebSocket error:', err);
    };
  } catch (e) {
    console.error('[RusTTY] Failed to connect WebSocket:', e);
  }
}

const IPC = {
  send(message) {
    const jsonStr = JSON.stringify(message);
    let sent = false;
    if (liveWs && liveWs.readyState === WebSocket.OPEN) {
      try {
        liveWs.send(jsonStr);
        sent = true;
      } catch (e) {
        console.warn('[RusTTY] Error sending via WebSocket:', e);
      }
    }
    if (!sent && window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(jsonStr);
    }
  },

  /** Request configs from Rust on startup */
  requestConfig() { this.send({ type: 'get_config' }); },
  requestClientConfig() { this.send({ type: 'get_client_config' }); },
  requestDocPage(pageId) { this.send({ type: 'get_doc_page', page_id: pageId }); },
  requestIcmpCheck() { this.send({ type: 'icmp_check' }); },

  openTerminal(hostName) { this.send({ type: 'open_terminal', host_name: hostName }); },
  openBridgeTerminal(index) { this.send({ type: 'connect_bridge', index }); },
  quickConnect(data) { this.send({ type: 'quick_connect', data }); },

  saveHost(data, editIndex, hostId, targetFolderId) {
    this.send({ type: 'save_host', data, edit_index: editIndex ?? null, host_id: hostId ?? null, target_folder_id: targetFolderId ?? null });
  },
  deleteHost(index, hostId) {
    this.send({ type: 'delete_host', index: index ?? null, host_id: hostId ?? null });
  },
  reorderHosts(fromIndex, toIndex) { this.send({ type: 'reorder_hosts', from_index: fromIndex, to_index: toIndex }); },
  saveFolder(data) { this.send({ type: 'save_folder', ...data }); },
  deleteFolder(folderId, keepContents) { this.send({ type: 'delete_folder', folder_id: folderId, keep_contents: keepContents }); },
  moveHost(hostId, targetFolderId, targetIndex) { this.send({ type: 'move_host', host_id: hostId, target_folder_id: targetFolderId, target_index: targetIndex }); },
  reorderNodes(parentId, order) { this.send({ type: 'reorder_nodes', parent_id: parentId, order }); },
  saveBridge(data, editIndex) { this.send({ type: 'save_bridge', data, edit_index: editIndex ?? null }); },
  deleteBridge(index) { this.send({ type: 'delete_bridge', index }); },

  saveSetting(key, value) { this.send({ type: 'save_setting', key, value }); },
  saveCustomization(data) { this.send({ type: 'save_customization', data }); },
  deleteKeyword(index) { this.send({ type: 'delete_keyword', index }); },

  openUrl(url) { this.send({ type: 'open_url', url }); },
};

// Rust → Frontend callback (fallback nativo)
window.__rustCallback = function (dataStr) {
  // Quando o WebSocket está ativo, as mensagens já chegam por ele em tempo real
  if (liveWs && liveWs.readyState === WebSocket.OPEN) {
    return;
  }
  try {
    const msg = typeof dataStr === 'string' ? JSON.parse(dataStr) : dataStr;
    handleRustMessage(msg);
  } catch (e) {
    console.error('Failed to parse Rust message:', e);
  }
};

//  Application State 

const state = {
  currentView: 'home',
  currentDocPage: null,
  hostSearchQuery: '',

  // Data from Rust
  nodes: [],
  hosts: [],
  bridges: [],
  collapsedFolders: {},
  clientConfig: {},
  settingsSchema: [],
  icmpStatus: {},
  docPages: [],
  docContent: {},

  // Navegação pendente: aguarda config_data fresca antes de renderizar o destino
  // Evita o efeito de "F5" (render duplo: dados antigos → dados novos)
  pendingNavigateAfterConfig: null,

  // Form state
  hostForm: createHostForm(),
  bridgeForm: createBridgeForm(),
  quickConnectForm: createQuickConnectForm(),

  // Edit mode
  editingHostIndex: null,
  editingHostId: null,
  editingBridgeIndex: null,

  // Customization
  customizationMode: 'list',
  customizationEditIdx: null,
  kwForm: { keyword: '', color: '#FF7300', caseInsensitive: false },
  ipForm: { split: false, unifiedColor: '#FF7300', publicColor: '#34C759', privateColor: '#FF453A' },
  ipFormTarget: 'ipv4',
};

function createHostForm() {
  return {
    name: '', address: '', port: '22', username: '', password: '',
    allowDomain: false, enableIcmp: true, enableBridge: false,
    selectedBridge: null, legacySsh: false, icon: 'terminal', showPassword: false,
    targetFolderId: '', error: null
  };
}
function createBridgeForm() {
  return {
    name: '', address: '', port: '22', username: '', password: '',
    allowDomain: false, showPassword: false, error: null
  };
}
function createQuickConnectForm() {
  return {
    protocol: 'ssh', address: '', port: '22', username: '', password: '',
    allowDomain: false, showPassword: false, error: null
  };
}

//  Handle Messages from Rust 

function handleRustMessage(msg) {
  switch (msg.type) {
    case 'config_data':
      state.nodes = msg.nodes || [];
      state.hosts = msg.hosts || [];
      state.bridges = msg.bridges || [];

      // Se há uma navegação pendente (ex: após salvar host/bridge), executa agora
      // com os dados já atualizados — evita o render duplo com dados antigos.
      if (state.pendingNavigateAfterConfig) {
        const dest = state.pendingNavigateAfterConfig;
        state.pendingNavigateAfterConfig = null;
        navigate(dest);
      } else {
        if (state.currentView === 'home' && !isDraggingHost) renderView();
        if (state.currentView === 'bridges') renderView();
      }

      IPC.requestIcmpCheck();
      break;

    case 'client_config_data':
      state.clientConfig = msg.data || {};
      // Protege o schema: nunca sobrescreve com array vazio se já tínhamos dados
      if (msg.settings_schema && msg.settings_schema.length > 0) {
        state.settingsSchema = msg.settings_schema;
      }
      if (msg.doc_pages && msg.doc_pages.length > 0) {
        state.docPages = msg.doc_pages;
      }
      if (state.currentView === 'settings') {
        const activeEl = document.activeElement;
        const isEditingField = activeEl && (activeEl.tagName === 'INPUT' || activeEl.tagName === 'SELECT') && !activeEl.classList.contains('slider');
        if (!isEditingField) {
          renderView();
        }
      }
      if (state.currentView === 'customization') renderView();
      break;

    case 'icmp_results':
      state.icmpStatus = msg.data || {};
      if (state.currentView === 'home') updateIcmpIndicators();
      break;

    case 'doc_page_content':
      state.docContent[msg.page_id] = msg.content;
      if (state.currentView === 'documentation') renderDocContent();
      break;

    case 'operation_result':
      if (msg.success) {
        Toast.show(msg.message || 'Operação realizada com sucesso.', 'success');

        // Navega de volta conforme a view atual
        if (state.currentView === 'new-host') {
          // Define navegação pendente: só acontece quando a config fresca chegar
          // Evita render com dados antigos (efeito "F5")
          state.pendingNavigateAfterConfig = 'home';
          IPC.requestConfig();
        } else if (state.currentView === 'new-bridge') {
          state.pendingNavigateAfterConfig = 'bridges';
          IPC.requestConfig();
        } else if (state.currentView === 'home' || state.currentView === 'bridges') {
          // Delete de host/bridge a partir da tela de listagem (via modal de confirmação)
          // A view já é a correta — basta atualizar os dados e re-renderizar no mesmo lugar
          state.pendingNavigateAfterConfig = state.currentView;
          IPC.requestConfig();
        } else if (state.currentView === 'customization') {
          // Formulário de keyword/ip foi confirmado — volta para a lista
          // A config já chegou via client_config_data antes do operation_result
          state.customizationMode = 'list';
          state.customizationEditIdx = null;
          renderView();
        }
        // Para save_setting, a resposta já veio como client_config_data — sem navegação
      } else {
        Toast.show(msg.error || 'Erro na operação.', 'error');
        if (state.currentView === 'new-host') {
          state.hostForm.error = msg.error;
          renderView();
        } else if (state.currentView === 'new-bridge') {
          state.bridgeForm.error = msg.error;
          renderView();
        } else if (state.currentView === 'quick-connect') {
          state.quickConnectForm.error = msg.error;
          renderView();
        } else if (state.currentView === 'customization') {
          // Erro ao salvar customização — mostra no toast (já feito) e permanece na view
          renderView();
        }
      }
      break;

    case 'terminal_error':
      Toast.show(msg.error || 'Erro ao abrir terminal.', 'error');
      break;

    case 'update_notification':
      showUpdateCelebrationModal({
        title: msg.title || 'RusTTY foi Atualizado!',
        message: msg.message || 'Uma nova versão do RusTTY foi baixada e instalada em segundo plano com sucesso!',
        version: msg.version || ''
      });
      break;
  }
}

//  Navigation / Router 

function findHostFolderId(nodes, hostId) {
  if (!nodes || !hostId) return '';
  for (const node of nodes) {
    if (node.type === 'folder') {
      if (node.children) {
        for (const child of node.children) {
          if (child.type === 'host' && (child.data.id === hostId || child.data.name === hostId)) {
            return node.id;
          }
          if (child.type === 'folder' && child.children) {
            for (const sub of child.children) {
              if (sub.type === 'host' && (sub.data.id === hostId || sub.data.name === hostId)) {
                return child.id;
              }
            }
          }
        }
      }
    }
  }
  return '';
}

function navigate(view, params = {}) {
  // Reset context menu
  closeContextMenu();

  state.currentView = view;

  // Prepare forms when entering form views
  if (view === 'new-host' && (params.editIndex != null || params.hostId != null)) {
    state.editingHostIndex = params.editIndex ?? null;
    state.editingHostId = params.hostId ?? null;
    const host = params.hostId
      ? state.hosts.find(h => h.id === params.hostId)
      : (params.editIndex != null ? state.hosts[params.editIndex] : null);
    if (host) {
      state.hostForm = {
        name: host.name, address: host.address, port: String(host.port),
        username: host.username, password: '', allowDomain: host.allow_domain || false,
        enableIcmp: host.enable_icmp ?? true, enableBridge: !!host.bridge_id,
        selectedBridge: host.bridge_id || null, legacySsh: host.legacy_ssh || false,
        icon: host.icon || 'terminal',
        targetFolderId: findHostFolderId(state.nodes, host.id || host.name),
        showPassword: false, error: null,
      };
    }
  } else if (view === 'new-host') {
    state.editingHostIndex = null;
    state.editingHostId = null;
    state.hostForm = createHostForm();
  }

  if (view === 'new-bridge' && params.editIndex != null) {
    state.editingBridgeIndex = params.editIndex;
    const bridge = state.bridges[params.editIndex];
    if (bridge) {
      state.bridgeForm = {
        name: bridge.name, address: bridge.address, port: String(bridge.port),
        username: bridge.username, password: '', allowDomain: bridge.allow_domain || false,
        showPassword: false, error: null,
      };
    }
  } else if (view === 'new-bridge') {
    state.editingBridgeIndex = null;
    state.bridgeForm = createBridgeForm();
  }

  if (view === 'quick-connect') {
    state.quickConnectForm = createQuickConnectForm();
  }

  if (view === 'documentation') {
    state.currentDocPage = params.pageId || (state.docPages[0]?.id ?? 'introduction');
    if (!state.docContent[state.currentDocPage]) {
      IPC.requestDocPage(state.currentDocPage);
    }
  }

  if (view === 'customization') {
    // Garante dados frescos do backend ao entrar na tela de personalização
    IPC.requestClientConfig();
    if (params.mode) {
      state.customizationMode = params.mode;
      state.customizationEditIdx = params.editIdx ?? null;
      if (params.mode === 'edit-keyword' && params.editIdx != null) {
        const kw = state.clientConfig.customization_data?.keywords?.[params.editIdx];
        if (kw) {
          state.kwForm = { keyword: kw.keyword, color: kw.color, caseInsensitive: kw.case_insensitive };
        }
      } else if (params.mode === 'edit-keyword') {
        state.kwForm = { keyword: '', color: '#FF7300', caseInsensitive: false };
      }
      if (params.mode === 'edit-ipv4' || params.mode === 'edit-ipv6') {
        state.ipFormTarget = params.mode === 'edit-ipv4' ? 'ipv4' : 'ipv6';
        const ipData = state.clientConfig.customization_data?.[state.ipFormTarget];
        if (ipData) {
          if (ipData.Unified) {
            state.ipForm = { split: false, unifiedColor: ipData.Unified, publicColor: '#34C759', privateColor: '#FF453A' };
          } else if (ipData.Split) {
            state.ipForm = { split: true, unifiedColor: '#FF7300', publicColor: ipData.Split.public, privateColor: ipData.Split.private };
          }
        } else {
          state.ipForm = { split: false, unifiedColor: '#FF7300', publicColor: '#34C759', privateColor: '#FF453A' };
        }
      }
    } else {
      state.customizationMode = 'list';
    }
  }

  if (view === 'settings') {
    // Garante dados frescos do backend ao entrar em configurações
    IPC.requestClientConfig();
  }

  if (view !== 'home') {
    stopHostSearchAnimation();
  }

  renderSidebar();
  renderView();
}

// Render Engine 

function renderSidebar() {
  const nav = document.getElementById('sidebar-nav');
  const isDocView = state.currentView === 'documentation';

  if (isDocView) {
    // Documentation sidebar with pages list
    let html = `
      <div class="sidebar-section-label">Documentação</div>
      ${state.docPages.map(p => `
        <button type="button" class="nav-item ${state.currentDocPage === p.id ? 'active' : ''}"
                onclick="navigate('documentation', { pageId: '${p.id}' })">
          <span>${escHtml(p.title)}</span>
        </button>
      `).join('')}
      <div style="height: 16px"></div>
      <button type="button" class="nav-item" onclick="navigate('home')">
        ${icon('undo', 18)}
        <span>Voltar ao App</span>
      </button>
    `;
    nav.innerHTML = html;
    return;
  }

  const items = [
    { id: 'home', icon: 'home', label: 'Início' },
    { id: 'bridges', icon: 'network', label: 'Pontes' },
    { id: 'settings', icon: 'settings', label: 'Configurações' },
    { id: 'customization', icon: 'paintbrush', label: 'Personalização' },
    { id: 'documentation', icon: 'book-open', label: 'Documentação' },
  ];

  nav.innerHTML = items.map(item => {
    const isActive = state.currentView === item.id ||
      (item.id === 'home' && (state.currentView === 'new-host' || state.currentView === 'quick-connect')) ||
      (item.id === 'bridges' && state.currentView === 'new-bridge');
    return `
      <button type="button" class="nav-item ${isActive ? 'active' : ''}"
              onclick="navigate('${item.id}')">
        <span class="nav-icon">${icon(item.icon, 20)}</span>
        <span>${item.label}</span>
      </button>
    `;
  }).join('');
}

function renderView() {
  const container = document.getElementById('view-container');
  let html = '';

  switch (state.currentView) {
    case 'home': html = viewHome(); break;
    case 'new-host': html = viewNewHost(); break;
    case 'bridges': html = viewBridges(); break;
    case 'new-bridge': html = viewNewBridge(); break;
    case 'quick-connect': html = viewQuickConnect(); break;
    case 'settings': html = viewSettings(); break;
    case 'customization': html = viewCustomization(); break;
    case 'documentation': html = viewDocumentation(); break;
    default: html = viewHome();
  }

  container.innerHTML = `<div class="animate-fade-in">${html}</div>`;
  bindViewEvents();
}

//  View: Home 

// ─── View: Home ──────────────────────────────────────────────────────────────

function viewHome() {
  const query = (state.hostSearchQuery || '').trim().toLowerCase();
  const hasItems = state.nodes.length > 0 || state.hosts.length > 0;

  // Filtra recursivamente por nome, endereço ou usuário em todas as pastas e subpastas
  const filteredHosts = query
    ? state.hosts.filter(h =>
      (h.name && h.name.toLowerCase().includes(query)) ||
      (h.address && h.address.toLowerCase().includes(query)) ||
      (h.username && h.username.toLowerCase().includes(query))
    )
    : state.hosts;

  let hostCardsHtml = '';
  if (!hasItems) {
    hostCardsHtml = `
      <div class="empty-state">
        ${icon('server', 48)}
        <div class="empty-state-title">Nenhum host ou pasta cadastrado</div>
        <div class="empty-state-text">Clique em "Novo Host" ou "Nova Pasta" para começar a organizar seu ambiente.</div>
        <div class="mt-4" style="display: flex; gap: var(--sp-2); justify-content: center;">
          <button type="button" class="btn btn--secondary" onclick="openCreateFolderModal(null)">
            ${icon('folder-plus', 16)} Nova Pasta
          </button>
          <button type="button" class="btn btn--primary" onclick="navigate('new-host')">
            ${icon('plus', 16)} Novo Host
          </button>
        </div>
      </div>
    `;
  } else if (query) {
    if (filteredHosts.length === 0) {
      hostCardsHtml = `
        <div class="empty-state animate-fade-in">
          ${icon('search', 44)}
          <div class="empty-state-title">Nenhum host encontrado</div>
          <div class="empty-state-text">Nenhum servidor corresponde à busca "<strong>${escHtml(state.hostSearchQuery)}</strong>" em nenhuma das pastas.</div>
          <button type="button" class="btn btn--secondary mt-4" onclick="clearHostSearch()">
            ${icon('undo', 16)} Limpar Busca
          </button>
        </div>
      `;
    } else {
      hostCardsHtml = `
        <div class="host-list" id="host-list">
          ${filteredHosts.map(h => hostCard(h, true, h.folder_path)).join('')}
        </div>
      `;
    }
  } else {
    // Exibição padrão hierárquica em árvore (Pastas, Subpastas e Hosts da raiz)
    hostCardsHtml = `
      <div class="host-list" id="host-list"
           ondragover="onRootDragOver(event)"
           ondrop="onRootDrop(event)">
        ${renderNodeTree(state.nodes)}
      </div>
    `;
  }

  // Barra de busca interativa
  const searchSection = hasItems || query ? `
    <div class="host-search-wrapper">
      <div class="host-search-container" id="host-search-container">
        <div class="host-search-icon-box" id="host-search-icon-box" title="Buscar host">
          <span class="host-search-icon active" data-icon="search">${icon('search', 18)}</span>
          <span class="host-search-icon" data-icon="folder-search">${icon('folder-search', 18)}</span>
          <span class="host-search-icon" data-icon="book-search">${icon('book-search', 18)}</span>
        </div>
        <div class="host-search-input-wrapper">
          <input type="text"
                 class="host-search-input"
                 id="host-search-input"
                 value="${escAttr(state.hostSearchQuery || '')}"
                 oninput="onHostSearchInput(event)"
                 onfocus="onHostSearchFocus()"
                 onblur="onHostSearchBlur()"
                 autocomplete="off"
                 spellcheck="false"
                 placeholder="" />
          <div class="host-search-placeholder ${state.hostSearchQuery ? 'is-hidden' : ''}" id="host-search-placeholder">
            <span class="host-search-placeholder-prefix">Buscar host </span>
            <span class="host-search-wave-text" id="host-search-wave-text"></span>
            <span class="host-search-cursor">|</span>
          </div>
        </div>
        ${state.hostSearchQuery ? `
          <button type="button" class="host-search-clear-btn" onclick="clearHostSearch()" title="Limpar busca">
            ${icon('x', 14)}
          </button>
        ` : ''}
      </div>
      ${query ? `
        <div class="host-search-meta">
          <span class="host-search-count">${filteredHosts.length} de ${state.hosts.length} ${state.hosts.length === 1 ? 'host encontrado' : 'hosts encontrados em todas as pastas'}</span>
          <button type="button" class="host-search-clear-link" onclick="clearHostSearch()">Limpar filtro</button>
        </div>
      ` : ''}
    </div>
  ` : '';

  return `
    <div class="view-inner view-inner--wide">
      <div class="page-header">
        <div class="page-header-left">
          <span class="page-title-icon">${icon('server', 28)}</span>
          <div>
            <h1 class="page-title">Seus Hosts</h1>
            <p class="page-subtitle">Gerencie e organize seus servidores em pastas e subpastas.</p>
          </div>
        </div>
        <div class="page-header-actions">
          <button type="button" class="btn btn--secondary" onclick="navigate('quick-connect')">
            ${icon('plug')} Conexão Rápida
          </button>
          <button type="button" class="btn btn--secondary" onclick="openCreateFolderModal(null)">
            ${icon('folder-plus')} Nova Pasta
          </button>
          <button type="button" class="btn btn--primary" onclick="navigate('new-host')">
            ${icon('plus')} Novo Host
          </button>
        </div>
      </div>
      ${searchSection}
      ${hostCardsHtml}
    </div>
  `;
}

function renderNodeTree(nodes, level = 1, parentFolder = null) {
  if (!nodes || nodes.length === 0) return '';
  return nodes.map(node => {
    if (node.type === 'host') {
      return hostCard(node.data, false, parentFolder ? [parentFolder.name] : []);
    } else if (node.type === 'folder') {
      return folderCard(node, level, parentFolder);
    }
    return '';
  }).join('');
}

function folderCard(folder, level = 1, parentFolder = null) {
  const isCollapsed = !!state.collapsedFolders[folder.id];
  const folderColor = folder.color || '#FF7300';
  const folderIcon = folder.icon || 'folder';

  // Contagem recursiva de hosts nesta pasta e suas subpastas
  let directHosts = 0;
  let subfolders = 0;
  let totalHosts = 0;

  function countRecursive(f) {
    if (!f.children) return;
    f.children.forEach(c => {
      if (c.type === 'host') {
        totalHosts++;
      } else if (c.type === 'folder') {
        countRecursive(c);
      }
    });
  }
  if (folder.children) {
    folder.children.forEach(c => {
      if (c.type === 'host') directHosts++;
      else if (c.type === 'folder') subfolders++;
    });
    countRecursive(folder);
  }

  const badgeText = level === 1
    ? `${totalHosts} ${totalHosts === 1 ? 'host' : 'hosts'}${subfolders > 0 ? ` • ${subfolders} ${subfolders === 1 ? 'subpasta' : 'subpastas'}` : ''}`
    : `${directHosts} ${directHosts === 1 ? 'host' : 'hosts'}`;

  // Se nível 1, permite criar subpasta. Se nível 2, NÃO permite criar subpasta (limite estrito de 2 níveis)
  const canCreateSubfolder = level === 1;

  const childrenHtml = folder.children && folder.children.length > 0
    ? folder.children.map(child => {
        if (child.type === 'folder') {
          return `<div class="subfolder-wrapper">${folderCard(child, 2, folder)}</div>`;
        } else if (child.type === 'host') {
          return hostCard(child.data, false, [folder.name]);
        }
        return '';
      }).join('')
    : `<div class="folder-empty-drop-zone" data-folder-id="${folder.id}"
            ondragover="onFolderDragOver(event, '${folder.id}')"
            ondragleave="onFolderDragLeave(event, '${folder.id}')"
            ondrop="onFolderDrop(event, '${folder.id}')">Pasta vazia. Arraste hosts aqui ou use "Mover para..."</div>`;

  const isDraggable = !state.hostSearchQuery || !state.hostSearchQuery.trim();
  const parentId = parentFolder ? parentFolder.id : '';

  return `
    <div class="folder-card ${level === 2 ? 'folder-card--subfolder' : ''} ${isCollapsed ? 'is-collapsed' : ''}"
         data-folder-id="${folder.id}"
         data-parent-id="${escAttr(parentId)}"
         data-level="${level}"
         ${isDraggable ? 'draggable="true"' : ''}
         ondragstart="onFolderDragStart(event, '${folder.id}', '${escAttr(parentId)}')"
         ondragend="onFolderDragEnd(event)">
      <div class="folder-header" onclick="onFolderHeaderClick(event, '${folder.id}')"
           ondragover="onFolderDragOver(event, '${folder.id}')"
           ondragleave="onFolderDragLeave(event, '${folder.id}')"
           ondrop="onFolderDrop(event, '${folder.id}')">
        ${isDraggable ? `
          <div class="folder-drag-handle" title="Arraste para reordenar esta pasta">
            ${icon('grip-vertical', 16)}
          </div>
        ` : `
          <div class="folder-drag-handle folder-drag-handle--disabled" title="Limpe a busca para reorganizar">
            ${icon('grip-vertical', 16)}
          </div>
        `}
        <div class="folder-chevron">
          ${icon('chevron-right', 18)}
        </div>
        <div class="folder-icon" style="background: ${folderColor}20; color: ${folderColor};">
          ${icon(folderIcon, level === 1 ? 20 : 18)}
        </div>
        <div class="folder-title-box">
          <div class="folder-title-row">
            <span class="folder-name">${escHtml(folder.name)}</span>
            ${level === 2 ? `<span class="folder-badge">Subpasta</span>` : ''}
          </div>
          <div class="folder-badges">
            <span class="folder-badge folder-badge--accent">${badgeText}</span>
          </div>
        </div>
        <div class="folder-actions" onclick="event.stopPropagation()">
          ${canCreateSubfolder ? `
            <button type="button" class="btn btn--icon" onclick="openCreateFolderModal('${folder.id}', '${escAttr(folder.name)}')" title="Nova Subpasta">
              ${icon('plus', 16)}
            </button>
          ` : ''}
          <button type="button" class="btn btn--icon" onclick="openEditFolderModal('${folder.id}')" title="Editar Pasta">
            ${icon('edit', 16)}
          </button>
          <button type="button" class="btn btn--icon btn--icon-danger" onclick="confirmDeleteFolder('${folder.id}', '${escAttr(folder.name)}', ${totalHosts})" title="Excluir Pasta">
            ${icon('trash', 16)}
          </button>
        </div>
      </div>
      <div class="folder-content" data-folder-id="${folder.id}"
           ondragover="onFolderDragOver(event, '${folder.id}')"
           ondragleave="onFolderDragLeave(event, '${folder.id}')"
           ondrop="onFolderDrop(event, '${folder.id}')">
        ${childrenHtml}
      </div>
    </div>
  `;
}

function onFolderHeaderClick(e, folderId) {
  if (isDraggingFolder || isDraggingHost) return;
  if (e.target.closest('.folder-actions') || e.target.closest('.folder-drag-handle') || e.target.closest('button')) return;
  toggleFolderCollapse(folderId);
}

function toggleFolderCollapse(folderId) {
  state.collapsedFolders[folderId] = !state.collapsedFolders[folderId];
  const folderEl = document.querySelector(`.folder-card[data-folder-id="${folderId}"]`);
  if (folderEl) {
    if (state.collapsedFolders[folderId]) {
      folderEl.classList.add('is-collapsed');
    } else {
      folderEl.classList.remove('is-collapsed');
    }
  }
}

function hostCard(host, isFiltered = false, folderPath = null) {
  const hostId = host.id || host.name;
  const icmpState = state.icmpStatus[hostId] ?? state.icmpStatus[host.name];
  let iconClass = '';
  if (host.enable_icmp && state.clientConfig.global_icmp) {
    if (icmpState === true) iconClass = 'host-icon-wrapper--online';
    else if (icmpState === false) iconClass = 'host-icon-wrapper--offline';
  }

  const bridgeTag = host.bridge_id
    ? `<span class="bridge-indicator">${icon('network', 12)} Ponte</span>` : '';

  const pathBadge = (folderPath && folderPath.length > 0)
    ? `<span class="host-path-badge" title="Pasta: ${escAttr(folderPath.join(' › '))}">${icon('folder', 12)} ${escHtml(folderPath.join(' › '))}</span>`
    : (isFiltered ? `<span class="host-path-badge host-path-badge--root" title="Host na raiz">${icon('home', 12)} Raiz</span>` : '');

  const hostIcon = host.icon || 'terminal';
  const isDraggable = !isFiltered;

  return `
    <div class="host-item stagger-item ${isFiltered ? 'is-filtered' : ''}" data-host-id="${escAttr(hostId)}" ${isDraggable ? 'draggable="true"' : ''}
         ${isDraggable ? `ondragstart="onHostDragStart(event, '${escAttr(hostId)}')" ondragend="onHostDragEnd(event)"` : ''}
         onclick="onHostCardClick(event, '${escAttr(host.name)}')"
         oncontextmenu="showHostContextMenu(event, '${escAttr(hostId)}')">
      ${isDraggable ? `
        <div class="host-drag-handle" title="Arraste para mover ou reordenar">
          ${icon('grip-vertical', 16)}
        </div>
      ` : `
        <div class="host-drag-handle host-drag-handle--disabled" title="Limpe a busca para reorganizar">
          ${icon('grip-vertical', 16)}
        </div>
      `}
      <div class="host-icon-wrapper ${iconClass}">
        ${icon(hostIcon, 20)}
      </div>
      <div class="host-info">
        <div class="host-name">${escHtml(host.name)}</div>
        <div class="host-detail">${escHtml(host.username)}@${escHtml(host.address)}:${host.port}</div>
      </div>
      ${pathBadge ? `<div>${pathBadge}</div>` : ''}
      ${bridgeTag ? `<div class="host-meta">${bridgeTag}</div>` : ''}
      <div class="host-actions">
        <button type="button" class="btn btn--icon" onclick="event.stopPropagation(); openMoveHostModal('${escAttr(hostId)}', '${escAttr(host.name)}')" title="Mover para...">
          ${icon('folder-symlink', 16)}
        </button>
        <button type="button" class="btn btn--icon" onclick="event.stopPropagation(); navigate('new-host', { hostId: '${escAttr(hostId)}' })" title="Editar">
          ${icon('edit', 16)}
        </button>
        <button type="button" class="btn btn--icon btn--icon-danger" onclick="event.stopPropagation(); confirmDeleteHost('${escAttr(hostId)}', '${escAttr(host.name)}')" title="Excluir">
          ${icon('trash', 16)}
        </button>
      </div>
    </div>
  `;
}

function updateIcmpIndicators() {
  document.querySelectorAll('.host-item').forEach(el => {
    const hid = el.dataset.hostId;
    const host = state.hosts.find(h => h.id === hid || h.name === hid);
    if (!host || !host.enable_icmp || !state.clientConfig.global_icmp) return;
    const wrapper = el.querySelector('.host-icon-wrapper');
    if (!wrapper) return;
    const icmpState = state.icmpStatus[hid] ?? state.icmpStatus[host.name];
    wrapper.classList.remove('host-icon-wrapper--online', 'host-icon-wrapper--offline');
    if (icmpState === true) wrapper.classList.add('host-icon-wrapper--online');
    else if (icmpState === false) wrapper.classList.add('host-icon-wrapper--offline');
  });
}

//  View: New Host 

function viewNewHost() {
  const f = state.hostForm;
  const isEditing = state.editingHostIndex != null;

  const bridgeOptions = state.bridges.map(b =>
    `<option value="${b.id}" ${f.selectedBridge === b.id ? 'selected' : ''}>${escHtml(b.name)}</option>`
  ).join('');

  const bridgeSection = f.enableBridge
    ? (state.bridges.length === 0
      ? `<div class="error-box">${icon('alert-triangle', 16)} Nenhuma ponte cadastrada. Vá em "Pontes" para cadastrar.</div>`
      : `<div class="input-group mt-2">
           <label class="input-label">${icon('network')} Selecionar Ponte</label>
           <div class="select-wrapper">
             <select class="select" data-bind="hostForm.selectedBridge">${bridgeOptions}</select>
           </div>
         </div>`)
    : '';

  const passwordSection = isEditing
    ? `<div class="input-group">
         <label class="input-label">${icon('lock')} Senha oculta por segurança</label>
         <p class="input-hint">Para alterar a senha, remova este host e crie um novo.</p>
       </div>`
    : `<div class="input-group">
         <label class="input-label">${icon('lock')} Senha</label>
         <div class="input-row">
           <input class="input" type="${f.showPassword ? 'text' : 'password'}"
                  placeholder="Senha SSH (opcional)" data-bind="hostForm.password"
                  value="${escAttr(f.password)}" autocomplete="off">
           <button type="button" class="btn btn--icon" onclick="state.hostForm.showPassword = !state.hostForm.showPassword; renderView()">
             ${icon(f.showPassword ? 'eye-off' : 'eye', 18)}
           </button>
         </div>
         <p class="input-hint">Deixe em branco para usar autenticação por chave.</p>
       </div>`;

  const hasFolders = (state.nodes || []).some(n => n.type === 'folder');
  let folderOptions = `<option value="">Raiz (Sem pasta)</option>`;
  if (hasFolders) {
    state.nodes.forEach(n => {
      if (n.type === 'folder') {
        folderOptions += `<option value="${n.id}" ${f.targetFolderId === n.id ? 'selected' : ''}>📁 ${escHtml(n.name)}</option>`;
        if (n.children) {
          n.children.forEach(sub => {
            if (sub.type === 'folder') {
              folderOptions += `<option value="${sub.id}" ${f.targetFolderId === sub.id ? 'selected' : ''}>&nbsp;&nbsp;↳ 📁 ${escHtml(sub.name)}</option>`;
            }
          });
        }
      }
    });
  }

  const folderSection = hasFolders
    ? `<div class="input-group">
         <label class="input-label">${icon('folder')} Pasta de Destino</label>
         <div class="select-wrapper">
           <select class="select" data-bind="hostForm.targetFolderId">${folderOptions}</select>
         </div>
         <p class="input-hint">Escolha em qual pasta ou subpasta este host será organizado.</p>
       </div>`
    : '';

  const errorHtml = f.error ? `<div class="error-box">${icon('x', 16)} ${escHtml(f.error)}</div>` : '';

  return `
    <div class="view-inner">
      <button type="button" class="back-btn" onclick="navigate('home')">
        ${icon('undo', 16)} Voltar
      </button>

      <div class="page-header">
        <div class="page-header-left">
          <span class="page-title-icon">${icon('server-plus', 28)}</span>
          <h1 class="page-title">${isEditing ? 'Editar Host SSH' : 'Novo Host'}</h1>
        </div>
      </div>

      <div class="form-section">
        <!-- Seletor de Ícone do Host -->
        <div class="input-group">
          <label class="input-label">${icon('sparkles', 16)} Ícone do Host</label>
          <div class="host-icon-picker-container" id="host-icon-picker-container">
            <button type="button" class="host-icon-selector-trigger" id="host-icon-trigger"
                    onclick="toggleHostIconMenu(event)" aria-haspopup="true" aria-expanded="false"
                    title="Clique para escolher outro ícone para este host">
              <div class="host-icon-current-box" id="host-icon-current-box">
                ${icon(f.icon || 'terminal', 24)}
              </div>
              <div class="host-icon-trigger-info">
                <div class="host-icon-trigger-title-row">
                  <span class="host-icon-trigger-name" id="host-icon-current-name">${getHostIconLabel(f.icon || 'terminal')}</span>
                  <span class="host-icon-trigger-badge">Ícone em uso</span>
                </div>
                <span class="host-icon-trigger-hint">Clique para abrir o menu e trocar o ícone</span>
              </div>
              <div class="host-icon-trigger-action">
                <span class="host-icon-trigger-action-text">Alterar</span>
                <span class="host-icon-trigger-chevron">${icon('chevron-right', 18)}</span>
              </div>
            </button>

            <!-- Menu Suspenso Tema Escuro -->
            <div class="host-icon-dropdown-menu" id="host-icon-menu">
              <div class="host-icon-menu-header">
                <span class="host-icon-menu-title">${icon('palette', 14)} Escolha um ícone para o host</span>
                <span class="host-icon-menu-close" onclick="closeHostIconMenu(event)" title="Fechar">${icon('x', 14)}</span>
              </div>
              <div class="host-icon-grid" id="host-icon-grid">
                ${HOST_AVAILABLE_ICONS.map(item => `
                  <button type="button" class="host-icon-grid-item ${item.id === (f.icon || 'terminal') ? 'selected' : ''}"
                          onclick="selectHostIcon('${item.id}', event)"
                          title="${item.label}"
                          data-icon-id="${item.id}">
                    <span class="host-icon-grid-preview">${icon(item.id, 22)}</span>
                    <span class="host-icon-grid-name">${item.label}</span>
                  </button>
                `).join('')}
              </div>
            </div>
          </div>
        </div>

        <div class="input-group">
          <label class="input-label">${icon('tag')} Nome / Apelido</label>
          <input class="input" type="text" placeholder="Ex: Servidor Prod"
                 data-bind="hostForm.name" value="${escAttr(f.name)}" autofocus>
        </div>

        <div class="input-group">
          <label class="input-label">${icon('network')} Endereço</label>
          <input class="input input--mono" type="text"
                 placeholder="${f.allowDomain ? 'Ex: meu.servidor.com ou 192.168.1.1' : 'Ex: 192.168.1.1 (somente IP numérico)'}"
                 data-bind="hostForm.address" value="${escAttr(f.address)}">
          <div class="form-checkboxes mt-2">
            ${checkbox('hostForm.allowDomain', f.allowDomain, 'Permitir domínio')}
            ${checkbox('hostForm.enableIcmp', f.enableIcmp, 'Habilitar teste ICMP')}
            ${checkbox('hostForm.legacySsh', f.legacySsh, 'SSH Legacy (servidores antigos)')}
          </div>
        </div>

        <div class="input-group">
          <label class="input-label">${icon('plug')} Porta SSH</label>
          <input class="input input--narrow input--mono" type="text" placeholder="22"
                 data-bind="hostForm.port" value="${escAttr(f.port)}" maxlength="5">
        </div>

        <div class="input-group">
          <label class="input-label">${icon('user')} Usuário SSH</label>
          <input class="input" type="text" placeholder="Ex: root, admin"
                 data-bind="hostForm.username" value="${escAttr(f.username)}">
        </div>

        ${passwordSection}

        ${folderSection}

        <div class="input-group">
          ${checkbox('hostForm.enableBridge', f.enableBridge, 'Habilitar ponte (Jump Host)')}
          ${bridgeSection}
        </div>

        ${errorHtml}

        <div class="form-actions">
          <button type="button" class="btn btn--primary btn--lg" onclick="submitHostForm()">
            ${icon('save')} Salvar Host
          </button>
          <button type="button" class="btn btn--ghost btn--lg" onclick="navigate('home')">
            ${icon('x')} Cancelar
          </button>
        </div>

        <div class="security-note">
          ${icon('shield', 14)}
          <span>Dados armazenados criptografados (AES-256-GCM) em %AppData%\\ByVitor\\RusTTY</span>
        </div>
      </div>
    </div>
  `;
}

function submitHostForm() {
  const f = state.hostForm;
  f.error = null;

  if (!f.name.trim()) { f.error = 'O nome/apelido do host é obrigatório.'; renderView(); return; }
  if (!f.address.trim()) { f.error = 'O endereço é obrigatório.'; renderView(); return; }
  if (!f.username.trim()) { f.error = 'O nome de usuário é obrigatório.'; renderView(); return; }
  const port = parseInt(f.port, 10);
  if (!port || port < 1 || port > 65535) { f.error = 'Porta inválida (1–65535).'; renderView(); return; }

  IPC.saveHost({
    name: f.name.trim(), address: f.address.trim(), port,
    username: f.username.trim(), password: f.password,
    allow_domain: f.allowDomain, enable_icmp: f.enableBridge ? false : f.enableIcmp,
    bridge_id: f.enableBridge ? f.selectedBridge : null,
    legacy_ssh: f.legacySsh,
    icon: f.icon || 'terminal',
  }, state.editingHostIndex, state.editingHostId, f.targetFolderId || null);
}

function toggleHostIconMenu(e) {
  if (e) {
    e.preventDefault();
    e.stopPropagation();
  }
  const menu = document.getElementById('host-icon-menu');
  const trigger = document.getElementById('host-icon-trigger');
  if (!menu) return;
  const isOpen = menu.classList.contains('is-open');
  if (isOpen) {
    closeHostIconMenu();
  } else {
    menu.classList.add('is-open');
    if (trigger) {
      trigger.classList.add('is-active');
      trigger.setAttribute('aria-expanded', 'true');
    }
  }
}

function closeHostIconMenu(e) {
  if (e) {
    e.preventDefault();
    e.stopPropagation();
  }
  const menu = document.getElementById('host-icon-menu');
  const trigger = document.getElementById('host-icon-trigger');
  if (menu) {
    menu.classList.remove('is-open');
  }
  if (trigger) {
    trigger.classList.remove('is-active');
    trigger.setAttribute('aria-expanded', 'false');
  }
}

function selectHostIcon(iconId, e) {
  if (e) {
    e.preventDefault();
    e.stopPropagation();
  }
  state.hostForm.icon = iconId;

  // Atualiza classe selected na grade do menu
  const grid = document.getElementById('host-icon-grid');
  if (grid) {
    grid.querySelectorAll('.host-icon-option').forEach(btn => {
      btn.classList.remove('selected');
    });
    const clickedBtn = grid.querySelector(`[onclick*="'${iconId}'"]`);
    if (clickedBtn) clickedBtn.classList.add('selected');
  }

  // Atualiza caixa e nome no gatilho do ícone atual
  const currentBox = document.getElementById('host-icon-current-box');
  const currentName = document.getElementById('host-icon-current-name');
  if (currentBox) currentBox.innerHTML = icon(iconId, 24);
  if (currentName) currentName.textContent = getHostIconLabel(iconId);

  // Fecha o menu após a escolha
  closeHostIconMenu();
}

//  View: Bridges 

function viewBridges() {
  const bridgeCards = state.bridges.length === 0
    ? `<div class="empty-state">
         ${icon('network', 48)}
         <div class="empty-state-title">Nenhuma ponte cadastrada</div>
         <div class="empty-state-text">Pontes (Jump Hosts) permitem acessar servidores em redes privadas.</div>
       </div>`
    : `<div class="host-list">
         ${state.bridges.map((b, i) => bridgeCard(b, i)).join('')}
       </div>`;

  return `
    <div class="view-inner view-inner--wide">
      <div class="page-header">
        <div class="page-header-left">
          <span class="page-title-icon">${icon('network', 28)}</span>
          <div>
            <h1 class="page-title">Suas Pontes</h1>
            <p class="page-subtitle">Jump Hosts para acessar redes privadas.</p>
          </div>
        </div>
        <div class="page-header-actions">
          <button type="button" class="btn btn--primary" onclick="navigate('new-bridge')">
            ${icon('plus')} Nova Ponte
          </button>
        </div>
      </div>
      ${bridgeCards}
    </div>
  `;
}

function bridgeCard(bridge, index) {
  return `
    <div class="host-item stagger-item"
         onclick="IPC.openBridgeTerminal(${index})"
         oncontextmenu="showBridgeContextMenu(event, ${index})">
      <div class="host-icon-wrapper">
        ${icon('network', 20)}
      </div>
      <div class="host-info">
        <div class="host-name">${escHtml(bridge.name)}</div>
        <div class="host-detail">${escHtml(bridge.username)}@${escHtml(bridge.address)}:${bridge.port}</div>
      </div>
      <div class="host-actions">
        <button type="button" class="btn btn--icon" onclick="event.stopPropagation(); navigate('new-bridge', { editIndex: ${index} })" title="Editar">
          ${icon('edit', 16)}
        </button>
        <button type="button" class="btn btn--icon btn--icon-danger" onclick="event.stopPropagation(); confirmDeleteBridge(${index})" title="Excluir">
          ${icon('trash', 16)}
        </button>
      </div>
    </div>
  `;
}

//  View: New Bridge 

function viewNewBridge() {
  const f = state.bridgeForm;
  const isEditing = state.editingBridgeIndex != null;

  const passwordSection = isEditing
    ? `<div class="input-group">
         <label class="input-label">${icon('lock')} Senha oculta por segurança</label>
         <p class="input-hint">Para alterar a senha, remova esta ponte e crie uma nova.</p>
       </div>`
    : `<div class="input-group">
         <label class="input-label">${icon('lock')} Senha</label>
         <div class="input-row">
           <input class="input" type="${f.showPassword ? 'text' : 'password'}"
                  placeholder="Senha SSH (opcional)" data-bind="bridgeForm.password"
                  value="${escAttr(f.password)}" autocomplete="off">
           <button type="button" class="btn btn--icon" onclick="state.bridgeForm.showPassword = !state.bridgeForm.showPassword; renderView()">
             ${icon(f.showPassword ? 'eye-off' : 'eye', 18)}
           </button>
         </div>
         <p class="input-hint">Deixe em branco para usar autenticação por chave.</p>
       </div>`;

  const errorHtml = f.error ? `<div class="error-box">${icon('x', 16)} ${escHtml(f.error)}</div>` : '';

  return `
    <div class="view-inner">
      <button type="button" class="back-btn" onclick="navigate('bridges')">
        ${icon('undo', 16)} Voltar
      </button>

      <div class="page-header">
        <div class="page-header-left">
          <span class="page-title-icon">${icon('network', 28)}</span>
          <h1 class="page-title">${isEditing ? 'Editar Ponte SSH' : 'Nova Ponte'}</h1>
        </div>
      </div>

      <div class="form-section">
        <div class="input-group">
          <label class="input-label">${icon('monitor')} Apelido / Nome</label>
          <input class="input" type="text" placeholder="Ex: Ponte Primária"
                 data-bind="bridgeForm.name" value="${escAttr(f.name)}" autofocus>
        </div>

        <div class="input-group">
          <label class="input-label">${icon('network')} Endereço da Ponte</label>
          <input class="input input--mono" type="text"
                 placeholder="${f.allowDomain ? 'Ex: ponte.empresa.com ou 10.0.0.1' : 'Ex: 10.0.0.1 (somente IP numérico)'}"
                 data-bind="bridgeForm.address" value="${escAttr(f.address)}">
          <div class="form-checkboxes mt-2">
            ${checkbox('bridgeForm.allowDomain', f.allowDomain, 'Permitir domínio')}
          </div>
        </div>

        <div class="input-group">
          <label class="input-label">${icon('plug')} Porta SSH</label>
          <input class="input input--narrow input--mono" type="text" placeholder="22"
                 data-bind="bridgeForm.port" value="${escAttr(f.port)}" maxlength="5">
        </div>

        <div class="input-group">
          <label class="input-label">${icon('user')} Usuário SSH</label>
          <input class="input" type="text" placeholder="Ex: root, admin"
                 data-bind="bridgeForm.username" value="${escAttr(f.username)}">
        </div>

        ${passwordSection}
        ${errorHtml}

        <div class="form-actions">
          <button type="button" class="btn btn--primary btn--lg" onclick="submitBridgeForm()">
            ${icon('save')} Salvar Ponte
          </button>
          <button type="button" class="btn btn--ghost btn--lg" onclick="navigate('bridges')">
            ${icon('x')} Cancelar
          </button>
        </div>
      </div>
    </div>
  `;
}

function submitBridgeForm() {
  const f = state.bridgeForm;
  f.error = null;
  if (!f.name.trim()) { f.error = 'O nome/apelido da ponte é obrigatório.'; renderView(); return; }
  if (!f.address.trim()) { f.error = 'O endereço é obrigatório.'; renderView(); return; }
  if (!f.username.trim()) { f.error = 'O nome de usuário é obrigatório.'; renderView(); return; }
  const port = parseInt(f.port, 10);
  if (!port || port < 1 || port > 65535) { f.error = 'Porta inválida (1�65535).'; renderView(); return; }

  IPC.saveBridge({
    name: f.name.trim(), address: f.address.trim(), port,
    username: f.username.trim(), password: f.password,
    allow_domain: f.allowDomain,
  }, state.editingBridgeIndex);
}

//  View: Quick Connect 

function viewQuickConnect() {
  const f = state.quickConnectForm;
  const isSsh = f.protocol === 'ssh';
  const isTelnet = f.protocol === 'telnet';

  const authFields = (isSsh || isTelnet) ? `
    <div class="form-row">
      <div class="input-group">
        <label class="input-label">Usuário</label>
        <input class="input" type="text" placeholder="Ex: root"
               data-bind="quickConnectForm.username" value="${escAttr(f.username)}">
      </div>
      <div class="input-group">
        <label class="input-label">Senha</label>
        <input class="input" type="${f.showPassword ? 'text' : 'password'}"
               placeholder="Senha (opcional)" data-bind="quickConnectForm.password"
               value="${escAttr(f.password)}" autocomplete="off">
      </div>
    </div>
    ${isSsh ? `
      <div class="form-checkboxes">
        ${checkbox('quickConnectForm.allowDomain', f.allowDomain, 'Permitir resolução de domínio (DNS)')}
        ${checkbox('quickConnectForm.showPassword', f.showPassword, 'Exibir senha')}
      </div>` : ''}
  ` : '';

  const errorHtml = f.error ? `<div class="error-box">${icon('alert-triangle', 16)} ${escHtml(f.error)}</div>` : '';

  return `
    <div class="view-inner">
      <button type="button" class="back-btn" onclick="navigate('home')">
        ${icon('undo', 16)} Voltar
      </button>

      <div class="page-header">
        <div class="page-header-left">
          <span class="page-title-icon">${icon('plug', 28)}</span>
          <h1 class="page-title">Conexão Rápida</h1>
        </div>
      </div>

      <div class="form-section">
        <div class="input-group">
          <label class="input-label">Protocolo</label>
          <div class="segmented-control">
            <button type="button" class="segment ${f.protocol === 'ssh' ? 'active' : ''}"
                    onclick="state.quickConnectForm.protocol='ssh'; state.quickConnectForm.port='22'; renderView()">SSH</button>
            <button type="button" class="segment ${f.protocol === 'telnet' ? 'active' : ''}"
                    onclick="state.quickConnectForm.protocol='telnet'; state.quickConnectForm.port='23'; renderView()">Telnet</button>
            <button type="button" class="segment ${f.protocol === 'serial' ? 'active' : ''}"
                    onclick="state.quickConnectForm.protocol='serial'; state.quickConnectForm.port=''; renderView()">Serial</button>
          </div>
        </div>

        <div class="form-row">
          <div class="input-group" style="flex:3">
            <label class="input-label">Endereço / Host</label>
            <input class="input input--mono" type="text" placeholder="Ex: 192.168.1.100 ou meuservidor.com"
                   data-bind="quickConnectForm.address" value="${escAttr(f.address)}">
          </div>
          <div class="input-group" style="flex:1">
            <label class="input-label">Porta</label>
            <input class="input input--mono" type="text" placeholder="22"
                   data-bind="quickConnectForm.port" value="${escAttr(f.port)}" maxlength="5">
          </div>
        </div>

        ${authFields}
        ${errorHtml}

        <div class="form-actions">
          <button type="button" class="btn btn--primary btn--lg" onclick="submitQuickConnect()">
            ${icon('globe-lock')} Conectar
          </button>
        </div>
      </div>
    </div>
  `;
}

function submitQuickConnect() {
  const f = state.quickConnectForm;
  f.error = null;
  if (f.protocol !== 'ssh') { f.error = 'Telnet e Serial ainda estão em desenvolvimento.'; renderView(); return; }
  if (!f.address.trim()) { f.error = 'O endereço é obrigatório.'; renderView(); return; }
  if (!f.username.trim()) { f.error = 'O nome de usuário é obrigatório.'; renderView(); return; }
  const port = parseInt(f.port, 10);
  if (!port || port < 1 || port > 65535) { f.error = 'Porta inválida (1�65535).'; renderView(); return; }

  IPC.quickConnect({
    address: f.address.trim(), port, username: f.username.trim(),
    password: f.password || 'none', allow_domain: f.allowDomain,
  });
}

//  View: Settings 

function viewSettings() {
  const c = state.clientConfig;
  const schema = state.settingsSchema || [];

  // Group settings by category
  const groups = {};
  for (const s of schema) {
    if (!groups[s.category]) groups[s.category] = [];
    groups[s.category].push(s);
  }

  let html = `
    <div class="view-inner" style="max-width: 650px">
      <div class="page-header">
        <div class="page-header-left">
          <span class="page-title-icon">${icon('settings', 28)}</span>
          <h1 class="page-title">Configurações</h1>
        </div>
      </div>
  `;

  for (const [category, settings] of Object.entries(groups)) {
    html += `<div class="settings-group-title mt-2">${category.toUpperCase()}</div>
             <div class="settings-group mb-6">`;
    for (const s of settings) {
      const val = c[s.key] !== undefined ? c[s.key] : false;
      const labelHtml = s.label;

      if (s.setting_type === 'boolean') {
        html += settingsToggleRow(labelHtml, s.description, s.key, val);
      } else if (s.setting_type === 'number') {
        html += settingsInputRow(labelHtml, s.description, s.key, val, 'Número');
      } else if (s.setting_type === 'char' || s.setting_type === 'text') {
        html += settingsInputRow(labelHtml, s.description, s.key, val, 'Texto');
      }
    }
    html += `</div>`;
  }

  html += `
      <!-- About -->
      <div class="settings-group-title">SOBRE O CLIENTE</div>
      <div class="settings-group mb-6">
        <div class="about-grid">
          ${aboutRow('Nome do Cliente', 'RusTTY')}
          ${aboutRow('Versão do Cliente', 'v2.1.0')}
          ${aboutRow('Data da Versão', '20/09/2026')}
          ${aboutRow('Licença', 'GNU Affero General Public License v3')}
          ${aboutRow('Desenvolvedor', 'Vitor')}
          ${aboutRow('Co-desenvolvedor', ' ')}
          ${aboutRow('Website', 'byvitor.com.br/rustty')}
        </div>
      </div>

      <!-- Supporters -->
      <div class="settings-group-title">APOIADORES DO PROJETO</div>
      <div class="settings-group mb-6">
        <div class="card-body">
          <p class="text-sm text-tertiary mb-4">Pessoas, empresas e organizações que apoiam o desenvolvimento do RusTTY:</p>
          <p class="font-semibold">DeepCraft Network</p>
          <p class="text-sm text-muted mt-4">Se o RusTTY foi útil para você, para a sua empresa ou organização, considere apoiar o projeto!</p>
        </div>
      </div>
    </div>
  `;
  return html;
}

function settingsToggleRow(label, description, key, value) {
  return `
    <div class="settings-item">
      <div class="settings-item-info">
        <div class="settings-item-label">${label}</div>
        <div class="settings-item-description">${description}</div>
      </div>
      ${toggleSwitch(key, value)}
    </div>
  `;
}

function settingsInputRow(label, description, key, value, placeholder) {
  return `
    <div class="settings-item">
      <div class="settings-item-info">
        <div class="settings-item-label">${label}</div>
        <div class="settings-item-description">${description}</div>
      </div>
      <input class="input input--sm input--narrow" type="text" placeholder="${placeholder}"
             value="${escAttr(String(value))}" data-setting="${key}"
             onchange="IPC.saveSetting('${key}', this.value)">
    </div>
  `;
}

function settingsSliderRow(label, description, key, value, min, max) {
  return `
    <div class="settings-item">
      <div class="settings-item-info">
        <div class="settings-item-label">${label}</div>
        <div class="settings-item-description">${description}</div>
      </div>
      <div class="slider-wrapper">
        <input type="range" class="slider" min="${min}" max="${max}" value="${value}"
               data-setting="${key}"
               oninput="this.nextElementSibling.textContent = this.value"
               onchange="IPC.saveSetting('${key}', this.value)">
        <span class="slider-value">${value}</span>
      </div>
    </div>
  `;
}

function aboutRow(label, value) {
  return `<div class="about-row">
    <span class="about-label">${label}</span>
    <span class="about-value">${value}</span>
  </div>`;
}

//  View: Customization 

function viewCustomization() {
  const c = state.clientConfig;

  if (!c.enable_customization) {
    return `
      <div class="warning-center">
        ${icon('alert-triangle', 48)}
        <h3>Personalização Desativada</h3>
        <p>Habilite a personalização nas Configurações para editar temas e aparências.</p>
      </div>
    `;
  }

  switch (state.customizationMode) {
    case 'edit-keyword': return viewKeywordForm();
    case 'edit-ipv4': case 'edit-ipv6': return viewIpForm();
    default: return viewCustomizationList();
  }
}

function viewCustomizationList() {
  const cd = state.clientConfig.customization_data || {};
  const keywords = cd.keywords || [];

  const ipv4Desc = formatIpDesc(cd.ipv4);
  const ipv6Desc = formatIpDesc(cd.ipv6);

  const kwItems = keywords.length === 0
    ? '<p class="text-sm text-tertiary" style="padding-left:4px">Nenhuma palavra-chave personalizada.</p>'
    : keywords.map((kw, i) => `
        <div class="customization-item stagger-item">
          <div class="customization-item-icon" style="background: ${kw.color}15; color: ${kw.color}">
            ${icon('at-sign', 20)}
          </div>
          <div class="customization-item-info">
            <div class="customization-item-title" style="color: ${kw.color}">${escHtml(kw.keyword)}</div>
            <div class="customization-item-subtitle">${kw.color} · ${kw.case_insensitive ? 'Case Insensitive' : 'Case Sensitive'}</div>
          </div>
          <div class="customization-item-actions">
            <button type="button" class="btn btn--icon" onclick="navigate('customization', { mode: 'edit-keyword', editIdx: ${i} })">${icon('edit', 16)}</button>
            <button type="button" class="btn btn--icon btn--icon-danger" onclick="IPC.deleteKeyword(${i})">${icon('trash', 16)}</button>
          </div>
        </div>
      `).join('');

  return `
    <div class="view-inner" style="max-width: 700px">
      <div class="page-header">
        <div class="page-header-left">
          <span class="page-title-icon">${icon('paintbrush', 28)}</span>
          <h1 class="page-title">Personalização</h1>
        </div>
        <div class="page-header-actions">
          <button type="button" class="btn btn--primary" onclick="navigate('customization', { mode: 'edit-keyword' })">
            ${icon('heart-plus')} Nova personalização
          </button>
        </div>
      </div>

      <div class="settings-group-title">PADRÕES DE SISTEMA</div>
      <div style="display: flex; flex-direction: column; gap: 8px; margin-bottom: 24px">
        <div class="customization-item" onclick="navigate('customization', { mode: 'edit-ipv4' })" style="cursor:pointer">
          <div class="customization-item-icon" style="background: rgba(255,115,0,0.08); color: var(--color-accent)">
            ${icon('network', 20)}
          </div>
          <div class="customization-item-info">
            <div class="customization-item-title">Personalização de IPv4</div>
            <div class="customization-item-subtitle">${ipv4Desc}</div>
          </div>
          <button type="button" class="btn btn--icon">${icon('edit', 16)}</button>
        </div>

        <div class="customization-item" onclick="navigate('customization', { mode: 'edit-ipv6' })" style="cursor:pointer">
          <div class="customization-item-icon" style="background: rgba(10,132,255,0.08); color: var(--color-info)">
            ${icon('globe', 20)}
          </div>
          <div class="customization-item-info">
            <div class="customization-item-title">Personalização de IPv6</div>
            <div class="customization-item-subtitle">${ipv6Desc}</div>
          </div>
          <button type="button" class="btn btn--icon">${icon('edit', 16)}</button>
        </div>
      </div>

      <div class="settings-group-title">PALAVRAS-CHAVE PERSONALIZADAS</div>
      <div style="display: flex; flex-direction: column; gap: 8px">
        ${kwItems}
      </div>
    </div>
  `;
}

function formatIpDesc(ipData) {
  if (!ipData) return 'Padrão do sistema';
  if (ipData.Unified) return `Cor unificada: ${ipData.Unified}`;
  if (ipData.Split) return `Público: ${ipData.Split.public} | Privado: ${ipData.Split.private}`;
  return 'Padrão do sistema';
}

function viewKeywordForm() {
  const f = state.kwForm;
  const isEditing = state.customizationEditIdx != null;

  return `
    <div class="view-inner">
      <button type="button" class="back-btn" onclick="navigate('customization')">
        ${icon('undo', 16)} Voltar
      </button>

      <div class="page-header">
        <div class="page-header-left">
          <span class="page-title-icon">${icon('edit', 28)}</span>
          <h1 class="page-title">${isEditing ? 'Editar Palavra-Chave' : 'Nova Palavra-Chave'}</h1>
        </div>
      </div>

      <div class="form-section">
        <div class="input-group">
          <label class="input-label">Palavra-Chave</label>
          <input class="input" type="text" placeholder="Palavra ou texto (ex: ERROR, down)"
                 data-bind="kwForm.keyword" value="${escAttr(f.keyword)}" autofocus>
        </div>

        ${checkbox('kwForm.caseInsensitive', f.caseInsensitive, 'Case Insensitive (ignorar maiúsculas/minúsculas)')}

        <div class="input-group mt-4">
          <label class="input-label">Cor da Palavra-Chave (Hex)</label>
          <div class="input-row">
            <div class="color-preview" style="background: ${f.color}"></div>
            <input class="input input--mono" type="text" placeholder="Ex: #FF0000" style="width: 150px"
                   data-bind="kwForm.color" value="${escAttr(f.color)}"
                   oninput="document.querySelector('.color-preview').style.background = this.value">
          </div>
        </div>

        <div class="form-actions mt-6">
          <button type="button" class="btn btn--primary btn--lg" onclick="submitKeywordForm()">
            ${icon('save')} Salvar Personalização
          </button>
        </div>
      </div>
    </div>
  `;
}

function submitKeywordForm() {
  const f = state.kwForm;
  if (!f.keyword.trim()) return;
  // Não navega imediatamente: aguarda operation_result do Rust para confirmar o sucesso.
  // A navegação de volta para 'list' é controlada em handleRustMessage → operation_result.
  IPC.saveCustomization({
    type: 'keyword',
    edit_index: state.customizationEditIdx,
    keyword: f.keyword.trim(),
    color: f.color.trim(),
    case_insensitive: f.caseInsensitive,
  });
}

function viewIpForm() {
  const f = state.ipForm;
  const isV4 = state.ipFormTarget === 'ipv4';

  const colorFields = f.split
    ? `
      ${colorPickerField('Cor para IP Público (Hex)', f.publicColor, 'ipForm.publicColor')}
      ${colorPickerField('Cor para IP Privado (Hex)', f.privateColor, 'ipForm.privateColor')}
    `
    : colorPickerField('Cor Unificada para todos os IPs (Hex)', f.unifiedColor, 'ipForm.unifiedColor');

  return `
    <div class="view-inner">
      <button type="button" class="back-btn" onclick="navigate('customization')">
        ${icon('undo', 16)} Voltar
      </button>

      <div class="page-header">
        <div class="page-header-left">
          <span class="page-title-icon">${icon(isV4 ? 'network' : 'globe', 28)}</span>
          <h1 class="page-title">Editar ${isV4 ? 'IPv4' : 'IPv6'}</h1>
        </div>
      </div>

      <div class="form-section">
        ${checkbox('ipForm.split', f.split, 'Cores separadas para IPs públicos e locais')}
        <div class="mt-4">${colorFields}</div>

        <div class="form-actions mt-6">
          <button type="button" class="btn btn--primary btn--lg" onclick="submitIpForm()">
            ${icon('save')} Salvar Personalização
          </button>
        </div>
      </div>
    </div>
  `;
}

function submitIpForm() {
  const f = state.ipForm;
  // Não navega imediatamente: aguarda operation_result do Rust para confirmar o sucesso.
  // A navegação de volta para 'list' é controlada em handleRustMessage → operation_result.
  IPC.saveCustomization({
    type: 'ip',
    target: state.ipFormTarget,
    split: f.split,
    unified_color: f.unifiedColor,
    public_color: f.publicColor,
    private_color: f.privateColor,
  });
}

function colorPickerField(label, value, bindKey) {
  return `
    <div class="input-group mb-4">
      <label class="input-label">${label}</label>
      <div class="input-row">
        <div class="color-preview" style="background: ${value}" data-color-for="${bindKey}"></div>
        <input class="input input--mono" type="text" placeholder="Ex: #FF0000" style="width: 150px"
               data-bind="${bindKey}" value="${escAttr(value)}"
               oninput="document.querySelector('[data-color-for=\\'${bindKey}\\']').style.background = this.value">
      </div>
    </div>
  `;
}

//  View: Documentation 

function viewDocumentation() {
  const content = state.docContent[state.currentDocPage];
  const rendered = content ? renderMarkdown(content) : '<p class="text-muted">Carregando...</p>';

  return `
    <div class="view-inner view-inner--wide">
      <div class="doc-content">${rendered}</div>
    </div>
  `;
}

function renderDocContent() {
  const docDiv = document.querySelector('.doc-content');
  if (docDiv && state.docContent[state.currentDocPage]) {
    docDiv.innerHTML = renderMarkdown(state.docContent[state.currentDocPage]);
  }
}

/** Lightweight Markdown HTML renderer (no external lib) */
function renderMarkdown(md) {
  let html = md
    // Fenced code blocks
    .replace(/```(\w*)\n([\s\S]*?)```/g, (_, lang, code) =>
      `<pre><code class="lang-${lang}">${escHtml(code.trim())}</code></pre>`)
    // Headers
    .replace(/^#### (.+)$/gm, '<h4>$1</h4>')
    .replace(/^### (.+)$/gm, '<h3>$1</h3>')
    .replace(/^## (.+)$/gm, '<h2>$1</h2>')
    .replace(/^# (.+)$/gm, '<h1>$1</h1>')
    // Horizontal rules
    .replace(/^---$/gm, '<hr>')
    // Bold + Italic
    .replace(/\*\*\*(.+?)\*\*\*/g, '<strong><em>$1</em></strong>')
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    // Inline code
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    // Blockquotes
    .replace(/^> (.+)$/gm, '<blockquote>$1</blockquote>')
    // Unordered lists
    .replace(/^- (.+)$/gm, '<li>$1</li>')
    // Links
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank">$1</a>');

  // Wrap consecutive <li> in <ul>
  html = html.replace(/((?:<li>.*<\/li>\n?)+)/g, '<ul>$1</ul>');

  // Paragraphs: wrap lines that aren't block elements
  html = html.split('\n').map(line => {
    const trimmed = line.trim();
    if (!trimmed) return '';
    if (/^<(h[1-6]|ul|ol|li|pre|blockquote|hr|div|p)/.test(trimmed)) return trimmed;
    return `<p>${trimmed}</p>`;
  }).join('\n');

  // Merge consecutive blockquotes
  html = html.replace(/<\/blockquote>\n?<blockquote>/g, '<br>');

  return html;
}

//  Components 

function checkbox(bindKey, checked, label) {
  return `
    <label class="checkbox-wrapper">
      <input type="checkbox" ${checked ? 'checked' : ''} onchange="onCheckboxChange('${bindKey}', this.checked)">
      <span class="checkbox-box">
        ${icon('check', 14)}
      </span>
      <span class="checkbox-label">${label}</span>
    </label>
  `;
}

function onCheckboxChange(bindKey, isChecked) {
  const parts = bindKey.split('.');
  let obj = state;
  for (let i = 0; i < parts.length - 1; i++) {
    if (!obj[parts[i]]) obj[parts[i]] = {};
    obj = obj[parts[i]];
  }
  obj[parts[parts.length - 1]] = isChecked;

  // Se "Habilitar ponte" mudou, atualiza a view para exibir ou ocultar o seletor de ponte
  if (bindKey === 'hostForm.enableBridge') {
    if (isChecked && !state.hostForm.selectedBridge) {
      state.hostForm.selectedBridge = state.bridges[0]?.id ?? null;
    }
    renderView();
    return;
  }

  // Se "SSH Legacy" for marcado na criação ou edição de host
  if (bindKey === 'hostForm.legacySsh') {
    if (isChecked) {
      if (state.clientConfig && state.clientConfig.ignore_security_warnings) {
        obj[parts[parts.length - 1]] = true;
        return;
      }

      // Reverte no DOM temporariamente enquanto aguarda confirmação com countdown
      obj[parts[parts.length - 1]] = false;
      const chk = document.querySelector('input[type="checkbox"][onchange*="hostForm.legacySsh"]');
      if (chk) chk.checked = false;

      showSecurityWarningModal({
        title: 'Aviso de Segurança: SSH Legado',
        badgeText: 'PROTOCOLO ANTIGO',
        descriptionText: 'O modo <strong>SSH Legacy</strong> utiliza protocolos antigos de SSH que podem não ser seguros e expor a conexão. Utilize apenas se o host não possuir suporte aos padrões modernos de SSH.',
        risks: [],
        countdownSeconds: 5,
        confirmButtonText: 'Confirmar',
        onConfirm: () => {
          state.hostForm.legacySsh = true;
          const currentChk = document.querySelector('input[type="checkbox"][onchange*="hostForm.legacySsh"]');
          if (currentChk) currentChk.checked = true;
          Toast.show('Modo SSH Legado ativado para este host.', 'warning');
        },
        onCancel: () => {
          state.hostForm.legacySsh = false;
          const currentChk = document.querySelector('input[type="checkbox"][onchange*="hostForm.legacySsh"]');
          if (currentChk) currentChk.checked = false;
        }
      });
      return;
    } else {
      obj[parts[parts.length - 1]] = false;
      return;
    }
  }

  // Se divisão de IP público/privado mudou, atualiza a view para alternar os pickers
  if (bindKey === 'ipForm.split') {
    renderView();
    return;
  }

  // Se permitir domínio mudou no hostForm ou bridgeForm, atualiza o placeholder dinamicamente sem perder foco
  if (bindKey === 'hostForm.allowDomain') {
    const addrInput = document.querySelector('[data-bind="hostForm.address"]');
    if (addrInput) {
      addrInput.placeholder = isChecked
        ? 'Ex: meu.servidor.com ou 192.168.1.1'
        : 'Ex: 192.168.1.1 (somente IP numérico)';
    }
  } else if (bindKey === 'bridgeForm.allowDomain') {
    const addrInput = document.querySelector('[data-bind="bridgeForm.address"]');
    if (addrInput) {
      addrInput.placeholder = isChecked
        ? 'Ex: ponte.empresa.com ou 10.0.0.1'
        : 'Ex: 10.0.0.1 (somente IP numérico)';
    }
  } else if (bindKey === 'quickConnectForm.showPassword') {
    const passInput = document.querySelector('[data-bind="quickConnectForm.password"]');
    if (passInput) {
      passInput.type = isChecked ? 'text' : 'password';
    }
  }
}

function toggleCheckbox(bindKey) {
  const parts = bindKey.split('.');
  let obj = state;
  for (let i = 0; i < parts.length - 1; i++) obj = obj[parts[i]];
  const newVal = !obj[parts[parts.length - 1]];
  onCheckboxChange(bindKey, newVal);
}

function toggleSwitch(settingKey, checked) {
  return `
    <label class="toggle-switch">
      <input type="checkbox" ${checked ? 'checked' : ''} onchange="toggleSetting('${settingKey}', this.checked)">
      <span class="toggle-track"></span>
      <span class="toggle-knob"></span>
    </label>
  `;
}

function toggleSetting(key, isChecked) {
  const newVal = isChecked !== undefined ? isChecked : !state.clientConfig[key];

  // Intercepta a opção de desativar os avisos de segurança globais
  if (key === 'ignore_security_warnings') {
    if (newVal) {
      // Reverte visualmente no DOM enquanto aguarda confirmação com cooldown de 10s
      const switchEl = document.querySelector(`input[onchange*="${key}"]`);
      if (switchEl) switchEl.checked = false;

      showSecurityWarningModal({
        title: 'Desativar Avisos de Segurança?',
        badgeText: 'CONFIGURAÇÃO AVANÇADA',
        descriptionText: 'Ao desativar os avisos de segurança, o RusTTY não exibirá confirmações ou alertas de risco ao habilitar conexões antigas como o SSH Legado.',
        risks: [],
        countdownSeconds: 10,
        confirmButtonText: 'Confirmar',
        onConfirm: () => {
          state.clientConfig[key] = true;
          IPC.saveSetting(key, true);
          renderView();
          Toast.show('Avisos de segurança desativados.', 'warning');
        },
        onCancel: () => {
          state.clientConfig[key] = false;
          renderView();
        }
      });
      return;
    } else {
      // Ao desativar o bypass (reativando os avisos), salva diretamente
      state.clientConfig[key] = false;
      IPC.saveSetting(key, false);
      renderView();
      Toast.show('Avisos de segurança reativados.', 'success');
      return;
    }
  }

  state.clientConfig[key] = newVal;
  IPC.saveSetting(key, newVal);

  // If customization was toggled, update the sidebar without re-rendering the whole view
  if (key === 'enable_customization') {
    renderSidebar();
  }
}

// ─── Modals ──────────────────────────────────────────────────────────────────

let activeSecurityModalTimer = null;
let activeSecurityModalCancelCb = null;

function clearSecurityModalState() {
  if (activeSecurityModalTimer) {
    clearInterval(activeSecurityModalTimer);
    activeSecurityModalTimer = null;
  }
  if (activeSecurityModalCancelCb) {
    const cb = activeSecurityModalCancelCb;
    activeSecurityModalCancelCb = null;
    cb();
  }
}

function showModal(html, extraClass = '') {
  const overlay = document.getElementById('modal-overlay');
  const content = document.getElementById('modal-content');
  if (!overlay || !content) return;
  content.className = 'modal-content' + (extraClass ? ' ' + extraClass : '');
  content.innerHTML = html;
  overlay.classList.add('visible');
}

function hideModal() {
  clearSecurityModalState();
  const overlay = document.getElementById('modal-overlay');
  const content = document.getElementById('modal-content');
  if (overlay) overlay.classList.remove('visible');
  stopCelebrationConfetti();
  setTimeout(() => {
    if (content) {
      content.className = 'modal-content';
      content.innerHTML = '';
    }
  }, 250);
}

function showSecurityWarningModal({
  title,
  badgeText = 'AVISO',
  descriptionText,
  risks = [],
  countdownSeconds = 5,
  confirmButtonText = 'Confirmar',
  onConfirm,
  onCancel,
}) {
  // Limpa qualquer estado anterior
  clearSecurityModalState();
  activeSecurityModalCancelCb = onCancel || null;
  let remaining = countdownSeconds;

  const risksHtml = risks.length > 0 ? `
    <div class="security-risks-section">
      <div class="security-risks-title">
        ${icon('alert-triangle', 16)} Principais Riscos:
      </div>
      <ul class="security-risks-list">
        ${risks.map(r => `
          <li>
            <span class="risk-bullet"></span>
            <div class="risk-item-content">
              <strong>${r.title}:</strong> ${r.detail}
            </div>
          </li>
        `).join('')}
      </ul>
    </div>
  ` : '';

  const html = `
    <div class="modal-header modal-header--security">
      <div class="security-header-left">
        <div class="security-badge-icon">
          ${icon('shield-alert', 24)}
        </div>
        <div>
          <div class="security-badge-tag">${badgeText}</div>
          <h3 class="security-modal-title">${title}</h3>
        </div>
      </div>
      <button type="button" class="btn btn--icon" onclick="hideModal()">${icon('x', 18)}</button>
    </div>

    <div class="modal-body security-modal-body">
      <div class="security-warning-box">
        <div class="security-warning-desc">${descriptionText}</div>
      </div>

      ${risksHtml}
    </div>

    <div class="modal-actions security-modal-actions">
      <button type="button" class="btn btn--ghost" onclick="hideModal()">
        Cancelar
      </button>
      <button type="button" id="security-confirm-btn" class="btn btn--danger btn--danger-countdown" disabled onclick="onSecurityConfirmClicked()">
        <span id="security-confirm-label">${confirmButtonText} (${remaining}s)</span>
      </button>
    </div>
  `;

  window.onSecurityConfirmClicked = function () {
    const btn = document.getElementById('security-confirm-btn');
    if (!btn || btn.disabled) return;
    if (activeSecurityModalTimer) {
      clearInterval(activeSecurityModalTimer);
      activeSecurityModalTimer = null;
    }
    activeSecurityModalCancelCb = null;
    hideModal();
    if (typeof onConfirm === 'function') {
      onConfirm();
    }
  };

  showModal(html, 'modal-content--security');

  activeSecurityModalTimer = setInterval(() => {
    remaining--;
    const label = document.getElementById('security-confirm-label');
    const btn = document.getElementById('security-confirm-btn');

    if (!btn) {
      clearInterval(activeSecurityModalTimer);
      activeSecurityModalTimer = null;
      return;
    }

    if (remaining > 0) {
      if (label) label.textContent = `${confirmButtonText} (${remaining}s)`;
    } else {
      clearInterval(activeSecurityModalTimer);
      activeSecurityModalTimer = null;
      btn.disabled = false;
      if (label) label.textContent = confirmButtonText;
    }
  }, 1000);
}

// ─── Celebration Confetti System ─────────────────────────────────────────────
let confettiAnimId = null;
let confettiParticles = [];

function launchCelebrationConfetti(count = 65) {
  let canvas = document.getElementById('celebration-confetti-canvas');
  if (!canvas) {
    canvas = document.createElement('canvas');
    canvas.id = 'celebration-confetti-canvas';
    canvas.className = 'celebration-confetti-canvas';
    document.body.appendChild(canvas);
  }

  const dpr = window.devicePixelRatio || 1;
  canvas.width = window.innerWidth * dpr;
  canvas.height = window.innerHeight * dpr;
  const ctx = canvas.getContext('2d');
  ctx.scale(dpr, dpr);

  const colors = [
    '#FF3B30', // Vermelho
    '#FF453A',
    '#FFCC00', // Amarelo
    '#FFD60A',
    '#007AFF', // Azul
    '#0A84FF',
    '#34C759', // Verde
    '#30D158'
  ];

  const originX = window.innerWidth / 2;
  const originY = window.innerHeight * 0.45;

  for (let i = 0; i < count; i++) {
    const angle = Math.random() * Math.PI * 2;
    const speed = 4 + Math.random() * 9;
    confettiParticles.push({
      x: originX + (Math.random() - 0.5) * 80,
      y: originY + (Math.random() - 0.5) * 40,
      vx: Math.cos(angle) * speed * (0.8 + Math.random() * 0.5),
      vy: Math.sin(angle) * speed - (3 + Math.random() * 4),
      size: 5 + Math.random() * 7,
      color: colors[Math.floor(Math.random() * colors.length)],
      rotation: Math.random() * 360,
      rotationSpeed: (Math.random() - 0.5) * 12,
      shape: Math.random() > 0.4 ? 'rect' : 'circle',
      opacity: 1,
      decay: 0.008 + Math.random() * 0.012,
      gravity: 0.18 + Math.random() * 0.08,
      wobble: Math.random() * 10,
    });
  }

  if (!confettiAnimId) {
    animateConfetti(canvas, ctx);
  }
}

function animateConfetti(canvas, ctx) {
  ctx.clearRect(0, 0, window.innerWidth, window.innerHeight);

  for (let i = confettiParticles.length - 1; i >= 0; i--) {
    const p = confettiParticles[i];
    p.x += p.vx;
    p.y += p.vy;
    p.vy += p.gravity;
    p.vx *= 0.985;
    p.rotation += p.rotationSpeed;
    p.opacity -= p.decay;
    p.wobble += 0.1;

    if (p.opacity <= 0 || p.y > window.innerHeight + 50) {
      confettiParticles.splice(i, 1);
      continue;
    }

    ctx.save();
    ctx.globalAlpha = Math.max(0, p.opacity);
    ctx.translate(p.x + Math.sin(p.wobble) * 2, p.y);
    ctx.rotate((p.rotation * Math.PI) / 180);
    ctx.fillStyle = p.color;

    if (p.shape === 'rect') {
      ctx.fillRect(-p.size / 2, -p.size / 2, p.size, p.size * (0.6 + Math.cos(p.wobble) * 0.4));
    } else {
      ctx.beginPath();
      ctx.arc(0, 0, p.size / 2, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.restore();
  }

  if (confettiParticles.length > 0) {
    confettiAnimId = requestAnimationFrame(() => animateConfetti(canvas, ctx));
  } else {
    confettiAnimId = null;
    ctx.clearRect(0, 0, window.innerWidth, window.innerHeight);
    if (canvas && canvas.parentNode) {
      canvas.parentNode.removeChild(canvas);
    }
  }
}

function stopCelebrationConfetti() {
  if (confettiAnimId) {
    cancelAnimationFrame(confettiAnimId);
    confettiAnimId = null;
  }
  confettiParticles = [];
  const canvas = document.getElementById('celebration-confetti-canvas');
  if (canvas && canvas.parentNode) {
    canvas.parentNode.removeChild(canvas);
  }
}

// ─── Celebration Modal Trigger ───────────────────────────────────────────────
function showUpdateCelebrationModal(options = {}) {
  const title = options.title || 'RusTTY foi Atualizado!';
  const message = options.message || 'Uma nova versão do RusTTY foi baixada e instalada silenciosamente em segundo plano. Suas conexões e configurações estão seguras e prontas para voar!';
  const version = options.version || '1.2.0';

  const overlay = document.getElementById('modal-overlay');
  const content = document.getElementById('modal-content');
  if (!overlay || !content) return;

  content.className = 'modal-content modal-content--celebration';

  const formattedVersion = version ? (version.startsWith('v') ? version : `v${version}`) : 'v1.2.0';

  content.innerHTML = `
    <div class="update-celebration-wrapper">
      <div class="update-celebration-card" id="update-celebration-card">
        <button type="button" class="update-celebration-close" onclick="hideModal()" title="Fechar (ESC)">
          ${icon('x', 15)}
        </button>

        <div class="update-celebration-badge">
          <span class="update-badge-icon">${icon('sparkles', 13)}</span>
          <span>Atualização Concluída</span>
        </div>

        <div class="update-celebration-hero">
          <div class="update-celebration-hero-icon">
            ${icon('rocket', 30)}
          </div>
        </div>

        <h2 class="update-celebration-title">${escHtml(title)}</h2>

        <div class="update-celebration-meta">
          <span class="update-version-chip">${icon('check', 11)} ${escHtml(formattedVersion)}</span>
          <span class="update-meta-sep">•</span>
          <span>Instalada em segundo plano</span>
        </div>

        <p class="update-celebration-desc">${escHtml(message)}</p>

        <div class="update-celebration-actions">
          <button type="button" class="btn--celebrate" id="btn-update-yay" onclick="onCelebrateYayClick()">
            <span>Yay! Continuar</span>
            ${icon('arrow-right', 16)}
          </button>
        </div>
      </div>
    </div>
  `;

  overlay.classList.add('visible');
  launchCelebrationConfetti(55);
}

function onCelebrateYayClick() {
  const btn = document.getElementById('btn-update-yay');
  if (btn) {
    btn.style.transform = 'scale(0.96)';
    setTimeout(() => { if (btn) btn.style.transform = ''; }, 150);
  }

  // Disparo extra suave ao clicar em Yay!
  launchCelebrationConfetti(70);

  // Fecha o modal suavemente após curtir a animação
  setTimeout(() => {
    hideModal();
  }, 450);
}

// ─── Folder Modals & Management ─────────────────────────────────────────────

let currentFolderModalState = {
  parentId: null,
  folderId: null,
  name: '',
  icon: 'folder',
  color: '#FF7300'
};

function selectFolderModalIcon(iconName) {
  currentFolderModalState.icon = iconName;
  document.querySelectorAll('.folder-icon-btn').forEach(btn => {
    btn.classList.toggle('is-selected', btn.dataset.icon === iconName);
  });
}

function selectFolderModalColor(colorHex) {
  currentFolderModalState.color = colorHex;
  document.querySelectorAll('.folder-color-swatch').forEach(btn => {
    btn.classList.toggle('is-selected', btn.dataset.color === colorHex);
  });
}

function openCreateFolderModal(parentId = null, parentName = '') {
  currentFolderModalState = {
    parentId: parentId || null,
    folderId: null,
    name: '',
    icon: 'folder',
    color: '#FF7300'
  };

  const isSubfolder = !!parentId;
  const title = isSubfolder ? `Nova Subpasta em "${escHtml(parentName)}"` : 'Nova Pasta';
  const subtitle = isSubfolder
    ? 'Subpastas organizam hosts em um segundo nível hierárquico.'
    : 'Crie uma pasta principal para agrupar e personalizar seus hosts.';

  const iconButtons = FOLDER_AVAILABLE_ICONS.map(ic => `
    <button type="button" class="folder-icon-btn ${ic === currentFolderModalState.icon ? 'is-selected' : ''}"
            data-icon="${ic}" onclick="selectFolderModalIcon('${ic}')" title="${ic}">
      ${icon(ic, 20)}
    </button>
  `).join('');

  const colorButtons = FOLDER_COLORS.map(c => `
    <button type="button" class="folder-color-swatch ${c.value === currentFolderModalState.color ? 'is-selected' : ''}"
            data-color="${c.value}" style="background: ${c.value};" onclick="selectFolderModalColor('${c.value}')" title="${escAttr(c.name)}">
    </button>
  `).join('');

  showModal(`
    <div class="modal-header">
      <div class="modal-title">${icon('folder-plus', 22)} ${title}</div>
      <button type="button" class="btn btn--icon" onclick="hideModal()">${icon('x', 18)}</button>
    </div>
    <div class="modal-body">
      <p class="modal-desc">${subtitle}</p>
      
      <div class="input-group mt-3">
        <label class="input-label">${icon('edit', 14)} Nome da Pasta</label>
        <input class="input" type="text" id="folder-modal-name" placeholder="Ex: Produção, Bancos de Dados, AWS..." autofocus
               onkeydown="if(event.key === 'Enter') submitCreateFolder()">
      </div>

      <div class="input-group mt-3">
        <label class="input-label">${icon('sparkles', 14)} Cor da Pasta</label>
        <div class="folder-color-grid">
          ${colorButtons}
        </div>
      </div>

      <div class="input-group mt-3">
        <label class="input-label">${icon('folder', 14)} Ícone da Pasta</label>
        <div class="folder-icon-grid">
          ${iconButtons}
        </div>
      </div>
    </div>
    <div class="modal-actions">
      <button type="button" class="btn btn--ghost" onclick="hideModal()">Cancelar</button>
      <button type="button" class="btn btn--primary" onclick="submitCreateFolder()">
        ${icon('save', 16)} ${isSubfolder ? 'Criar Subpasta' : 'Criar Pasta'}
      </button>
    </div>
  `);

  setTimeout(() => {
    const input = document.getElementById('folder-modal-name');
    if (input) input.focus();
  }, 100);
}

function submitCreateFolder() {
  const input = document.getElementById('folder-modal-name');
  const name = input ? input.value.trim() : '';
  if (!name) {
    if (input) input.focus();
    Toast.show('Digite o nome da pasta.', 'warning');
    return;
  }

  IPC.saveFolder({
    parent_id: currentFolderModalState.parentId,
    name: name,
    icon: currentFolderModalState.icon,
    color: currentFolderModalState.color
  });

  hideModal();
  Toast.show(currentFolderModalState.parentId ? 'Subpasta criada com sucesso!' : 'Pasta criada com sucesso!', 'success');
}

function findFolderInNodes(nodes, folderId) {
  if (!nodes || !folderId) return null;
  for (const node of nodes) {
    if (node.type === 'folder') {
      if (node.id === folderId) return node;
      if (node.children) {
        const found = findFolderInNodes(node.children, folderId);
        if (found) return found;
      }
    }
  }
  return null;
}

function openEditFolderModal(folderId) {
  const folder = findFolderInNodes(state.nodes, folderId);
  if (!folder) return;

  currentFolderModalState = {
    folderId: folder.id,
    parentId: folder.parent_id || null,
    name: folder.name,
    icon: folder.icon || 'folder',
    color: folder.color || '#FF7300'
  };

  const iconButtons = FOLDER_AVAILABLE_ICONS.map(ic => `
    <button type="button" class="folder-icon-btn ${ic === currentFolderModalState.icon ? 'is-selected' : ''}"
            data-icon="${ic}" onclick="selectFolderModalIcon('${ic}')" title="${ic}">
      ${icon(ic, 20)}
    </button>
  `).join('');

  const colorButtons = FOLDER_COLORS.map(c => `
    <button type="button" class="folder-color-swatch ${c.value === currentFolderModalState.color ? 'is-selected' : ''}"
            data-color="${c.value}" style="background: ${c.value};" onclick="selectFolderModalColor('${c.value}')" title="${escAttr(c.name)}">
    </button>
  `).join('');

  showModal(`
    <div class="modal-header">
      <div class="modal-title">${icon('edit', 22)} Editar Pasta "${escHtml(folder.name)}"</div>
      <button type="button" class="btn btn--icon" onclick="hideModal()">${icon('x', 18)}</button>
    </div>
    <div class="modal-body">
      <div class="input-group">
        <label class="input-label">${icon('edit', 14)} Nome da Pasta</label>
        <input class="input" type="text" id="folder-modal-name" value="${escAttr(folder.name)}"
               onkeydown="if(event.key === 'Enter') submitEditFolder('${folder.id}')">
      </div>

      <div class="input-group mt-3">
        <label class="input-label">${icon('sparkles', 14)} Cor da Pasta</label>
        <div class="folder-color-grid">
          ${colorButtons}
        </div>
      </div>

      <div class="input-group mt-3">
        <label class="input-label">${icon('folder', 14)} Ícone da Pasta</label>
        <div class="folder-icon-grid">
          ${iconButtons}
        </div>
      </div>
    </div>
    <div class="modal-actions">
      <button type="button" class="btn btn--ghost" onclick="hideModal()">Cancelar</button>
      <button type="button" class="btn btn--primary" onclick="submitEditFolder('${folder.id}')">
        ${icon('save', 16)} Salvar Alterações
      </button>
    </div>
  `);

  setTimeout(() => {
    const input = document.getElementById('folder-modal-name');
    if (input) { input.focus(); input.select(); }
  }, 100);
}

function submitEditFolder(folderId) {
  const input = document.getElementById('folder-modal-name');
  const name = input ? input.value.trim() : '';
  if (!name) {
    if (input) input.focus();
    Toast.show('Digite o nome da pasta.', 'warning');
    return;
  }

  IPC.saveFolder({
    folder_id: folderId,
    name: name,
    icon: currentFolderModalState.icon,
    color: currentFolderModalState.color
  });

  hideModal();
  Toast.show('Pasta atualizada com sucesso!', 'success');
}

function confirmDeleteFolder(folderId, folderName, totalHosts) {
  if (totalHosts > 0) {
    showModal(`
      <div class="modal-header">
        <div class="modal-title modal-title--danger">${icon('alert-triangle', 22)} Excluir Pasta "${escHtml(folderName)}"?</div>
        <button type="button" class="btn btn--icon" onclick="hideModal()">${icon('x', 18)}</button>
      </div>
      <div class="modal-body">
        <p>Esta pasta contém <strong>${totalHosts} ${totalHosts === 1 ? 'host salvo' : 'hosts salvos'}</strong>.</p>
        <p class="mt-2">Você pode manter os seus hosts movendo-os com segurança para a raiz, ou excluir permanentemente a pasta junto com seus hosts.</p>
      </div>
      <div class="modal-actions" style="flex-direction: column; gap: var(--sp-2);">
        <button type="button" class="btn btn--primary btn--full" onclick="hideModal(); IPC.deleteFolder('${folderId}', true); Toast.show('Pasta excluída. Hosts movidos para a raiz.', 'info')">
          ${icon('folder-symlink', 16)} Manter Hosts (Mover para a Raiz)
        </button>
        <button type="button" class="btn btn--danger btn--full" onclick="hideModal(); IPC.deleteFolder('${folderId}', false); Toast.show('Pasta e hosts excluídos permanentemente.', 'warning')">
          ${icon('trash', 16)} Excluir pasta e todos os hosts
        </button>
        <button type="button" class="btn btn--ghost btn--full" onclick="hideModal()">
          Cancelar
        </button>
      </div>
    `);
  } else {
    showModal(`
      <div class="modal-header">
        <div class="modal-title modal-title--danger">${icon('alert-triangle', 22)} Excluir Pasta "${escHtml(folderName)}"?</div>
        <button type="button" class="btn btn--icon" onclick="hideModal()">${icon('x', 18)}</button>
      </div>
      <div class="modal-body">
        <p>Esta pasta está vazia. Tem certeza que deseja removê-la?</p>
      </div>
      <div class="modal-actions">
        <button type="button" class="btn btn--ghost" onclick="hideModal()">Cancelar</button>
        <button type="button" class="btn btn--danger" onclick="hideModal(); IPC.deleteFolder('${folderId}', true); Toast.show('Pasta removida.', 'info')">
          ${icon('trash', 16)} Excluir Pasta
        </button>
      </div>
    `);
  }
}

function openMoveHostModal(hostId, hostName) {
  const host = state.hosts.find(h => h.id === hostId || h.name === hostId);
  const currentPath = host && host.folder_path ? host.folder_path : [];

  let destinations = [];

  // Raiz
  const isCurrentlyRoot = currentPath.length === 0;
  destinations.push(`
    <button type="button" class="destination-item destination-item--root ${isCurrentlyRoot ? 'is-current' : ''}"
            onclick="executeMoveHost('${escAttr(hostId)}', null)">
      <div class="destination-icon" style="background: rgba(255, 115, 0, 0.15); color: var(--color-accent);">
        ${icon('home', 18)}
      </div>
      <div class="destination-info">
        <div class="destination-title">Raiz (Sem pasta)</div>
        <div class="destination-sub">${isCurrentlyRoot ? '✓ Local atual' : 'Mover para o nível principal'}</div>
      </div>
    </button>
  `);

  // Pastas e Subpastas
  if (state.nodes) {
    state.nodes.forEach(node => {
      if (node.type === 'folder') {
        const folderColor = node.color || '#FF7300';
        const folderIcon = node.icon || 'folder';
        const isCurrent = currentPath.length === 1 && currentPath[0] === node.name;
        destinations.push(`
          <button type="button" class="destination-item ${isCurrent ? 'is-current' : ''}"
                  onclick="executeMoveHost('${escAttr(hostId)}', '${node.id}')">
            <div class="destination-icon" style="background: ${folderColor}20; color: ${folderColor};">
              ${icon(folderIcon, 18)}
            </div>
            <div class="destination-info">
              <div class="destination-title">${escHtml(node.name)}</div>
              <div class="destination-sub">${isCurrent ? '✓ Local atual' : 'Pasta Principal'}</div>
            </div>
          </button>
        `);

        if (node.children) {
          node.children.forEach(sub => {
            if (sub.type === 'folder') {
              const subColor = sub.color || folderColor;
              const subIcon = sub.icon || 'folder';
              const isSubCurrent = currentPath.length === 2 && currentPath[0] === node.name && currentPath[1] === sub.name;
              destinations.push(`
                <button type="button" class="destination-item destination-item--subfolder ${isSubCurrent ? 'is-current' : ''}"
                        onclick="executeMoveHost('${escAttr(hostId)}', '${sub.id}')">
                  <div class="destination-icon" style="background: ${subColor}20; color: ${subColor};">
                    ${icon(subIcon, 16)}
                  </div>
                  <div class="destination-info">
                    <div class="destination-title">${escHtml(sub.name)}</div>
                    <div class="destination-sub">${isSubCurrent ? '✓ Local atual' : `Subpasta de ${escHtml(node.name)}`}</div>
                  </div>
                </button>
              `);
            }
          });
        }
      }
    });
  }

  showModal(`
    <div class="modal-header">
      <div class="modal-title">${icon('folder-symlink', 22)} Mover "${escHtml(hostName || 'Host')}"</div>
      <button type="button" class="btn btn--icon" onclick="hideModal()">${icon('x', 18)}</button>
    </div>
    <div class="modal-body">
      <p class="modal-desc">Selecione para qual pasta ou subpasta deseja mover este host:</p>
      <div class="destination-list">
        ${destinations.join('')}
      </div>
    </div>
    <div class="modal-actions">
      <button type="button" class="btn btn--ghost" onclick="hideModal()">Cancelar</button>
    </div>
  `);
}

function executeMoveHost(hostId, targetFolderId) {
  IPC.moveHost(hostId, targetFolderId || null, null);
  hideModal();
  Toast.show(targetFolderId ? 'Host movido para a pasta com sucesso!' : 'Host movido para a raiz!', 'success');
}

function confirmDeleteHost(identifier, optionalName = null) {
  let host = null;
  let hostId = null;
  let index = null;
  if (typeof identifier === 'number') {
    index = identifier;
    host = state.hosts[index];
    hostId = host ? host.id : null;
  } else {
    hostId = String(identifier);
    host = state.hosts.find(h => h.id === hostId || h.name === hostId);
    index = host ? state.hosts.indexOf(host) : null;
  }
  const name = optionalName || (host ? host.name : 'Host');

  showModal(`
    <div class="modal-header">
      <div class="modal-title modal-title--danger">${icon('alert-triangle', 22)} Deletar "${escHtml(name)}"?</div>
      <button type="button" class="btn btn--icon" onclick="hideModal()">${icon('x', 18)}</button>
    </div>
    <div class="modal-body">
      <p>Esta ação é permanente e irreversível.</p>
    </div>
    <div class="modal-actions">
      <button type="button" class="btn btn--ghost" onclick="hideModal()">Cancelar</button>
      <button type="button" class="btn btn--danger" onclick="hideModal(); IPC.deleteHost(${index != null ? index : 'null'}, '${escAttr(hostId || '')}')">
        ${icon('trash', 16)} Deletar permanentemente
      </button>
    </div>
  `);
}

function confirmDeleteBridge(index) {
  const bridge = state.bridges[index];
  if (!bridge) return;

  const affectedHosts = state.hosts.filter(h => h.bridge_id === bridge.id).length;
  const warningText = affectedHosts > 0
    ? `<p class="warning-text">${affectedHosts} host(s) utilizam esta ponte. Eles voltarão a conectar diretamente.</p>` : '';

  showModal(`
    <div class="modal-header">
      <div class="modal-title modal-title--danger">${icon('alert-triangle', 22)} Deletar "${escHtml(bridge.name)}"?</div>
      <button type="button" class="btn btn--icon" onclick="hideModal()">${icon('x', 18)}</button>
    </div>
    <div class="modal-body">
      <p>Esta ação é permanente e irreversível.</p>
      ${warningText}
    </div>
    <div class="modal-actions">
      <button type="button" class="btn btn--ghost" onclick="hideModal()">Cancelar</button>
      <button type="button" class="btn btn--danger" onclick="hideModal(); IPC.deleteBridge(${index})">
        ${icon('trash', 16)} ${affectedHosts > 0 ? 'Apagar mesmo assim' : 'Deletar permanentemente'}
      </button>
    </div>
  `);
}

// Toast System 

const Toast = {
  lastSignature: '',
  lastTimestamp: 0,

  show(message, type = 'info', duration = 4000) {
    const now = Date.now();
    const sig = `${type}::${message}`;
    // Deduplicação defensiva: impede toasts idênticos disparados em sequência rápida (< 1.5s)
    if (this.lastSignature === sig && (now - this.lastTimestamp < 1500)) {
      return;
    }
    this.lastSignature = sig;
    this.lastTimestamp = now;

    const container = document.getElementById('toast-container');
    if (!container) return;
    const toastEl = document.createElement('div');
    toastEl.className = `toast toast--${type}`;

    const icons = { success: 'check', error: 'alert-triangle', warning: 'alert-triangle', info: 'info' };
    toastEl.innerHTML = `${icon(icons[type] || 'info', 18)}<span>${escHtml(message)}</span>`;

    container.appendChild(toastEl);

    setTimeout(() => {
      toastEl.classList.add('toast-exit');
      setTimeout(() => toastEl.remove(), 300);
    }, duration);
  }
};

//  Context Menu 

function showHostContextMenu(e, hostId) {
  e.preventDefault();
  e.stopPropagation();
  const host = state.hosts.find(h => h.id === hostId || h.name === hostId);
  const hostName = host ? host.name : '';
  const isInFolder = host && host.folder_path && host.folder_path.length > 0;

  const menu = document.getElementById('context-menu');
  menu.innerHTML = `
    <button type="button" class="context-menu-item" onclick="closeContextMenu(); navigate('new-host', { hostId: '${escAttr(hostId)}' })">
      ${icon('edit', 16)} Editar host
    </button>
    <button type="button" class="context-menu-item" onclick="closeContextMenu(); openMoveHostModal('${escAttr(hostId)}', '${escAttr(hostName)}')">
      ${icon('folder-symlink', 16)} Mover para...
    </button>
    ${isInFolder ? `
      <button type="button" class="context-menu-item" onclick="closeContextMenu(); executeMoveHost('${escAttr(hostId)}', null)">
        ${icon('undo', 16)} Retirar da pasta (Mover para Raiz)
      </button>
    ` : ''}
    <div class="context-menu-separator"></div>
    <button type="button" class="context-menu-item context-menu-item--danger" onclick="closeContextMenu(); confirmDeleteHost('${escAttr(hostId)}', '${escAttr(hostName)}')">
      ${icon('trash', 16)} Remover host
    </button>
  `;
  positionContextMenu(menu, e);
}

function showBridgeContextMenu(e, index) {
  e.preventDefault();
  e.stopPropagation();
  const menu = document.getElementById('context-menu');
  menu.innerHTML = `
    <button type="button" class="context-menu-item" onclick="closeContextMenu(); navigate('new-bridge', { editIndex: ${index} })">
      ${icon('edit', 16)} Editar ponte
    </button>
    <div class="context-menu-separator"></div>
    <button type="button" class="context-menu-item context-menu-item--danger" onclick="closeContextMenu(); confirmDeleteBridge(${index})">
      ${icon('trash', 16)} Remover ponte
    </button>
  `;
  positionContextMenu(menu, e);
}

function positionContextMenu(menu, e) {
  menu.style.left = `${Math.min(e.clientX, window.innerWidth - 180)}px`;
  menu.style.top = `${Math.min(e.clientY, window.innerHeight - 100)}px`;
  menu.classList.add('visible');
}

function closeContextMenu() {
  document.getElementById('context-menu').classList.remove('visible');
}

//  Event Binding (Data-Bind System) 

function bindViewEvents() {
  // Bind all data-bind inputs/selects to state
  document.querySelectorAll('[data-bind]').forEach(el => {
    const bindKey = el.getAttribute('data-bind');

    const applyValue = (e) => {
      const parts = bindKey.split('.');
      let obj = state;
      for (let i = 0; i < parts.length - 1; i++) obj = obj[parts[i]];

      let value = e.target.type === 'checkbox' ? e.target.checked : e.target.value;

      // Port field: only allow digits
      if (parts[parts.length - 1] === 'port') {
        value = value.replace(/[^\d]/g, '').slice(0, 5);
        e.target.value = value;
      }

      obj[parts[parts.length - 1]] = value;
    };

    // <select> dispara 'change', inputs de texto disparam 'input'.
    // Registramos ambos para cobrir todos os casos sem duplicidade.
    if (el.tagName === 'SELECT') {
      el.addEventListener('change', applyValue);
    } else {
      el.addEventListener('input', applyValue);
    }
  });

  if (state.currentView === 'home') {
    initHostDragAndDrop();
    startHostSearchAnimation();
  } else {
    stopHostSearchAnimation();
  }
}

// ─── Drag and Drop & Node Reordering System ──────────────────────────────────

// ─── Drag and Drop & Node Reordering System ──────────────────────────────────

let isDraggingHost = false;
let draggedHostId = null;
let isDraggingFolder = false;
let draggedFolderId = null;
let draggedFolderParentId = null;

function onHostCardClick(e, hostName) {
  if (isDraggingHost || isDraggingFolder) return;
  if (e.target.closest('.host-actions') || e.target.closest('.host-drag-handle') || e.target.closest('button')) return;
  IPC.openTerminal(hostName);
}

function onHostDragStart(e, hostId) {
  if (e.target.closest('.host-actions') || e.target.closest('button')) {
    e.preventDefault();
    return;
  }
  draggedHostId = hostId;
  isDraggingHost = true;
  e.dataTransfer.effectAllowed = 'move';
  e.dataTransfer.setData('text/rus-host-id', hostId);
  e.dataTransfer.setData('text/plain', hostId);
  const card = e.currentTarget;
  if (card) {
    card.classList.add('is-dragging');
  }
}

function onHostDragEnd(e) {
  const card = e.currentTarget;
  if (card) {
    card.classList.remove('is-dragging');
  }
  document.querySelectorAll('.folder-card').forEach(f => f.classList.remove('is-drag-over', 'drag-over-top', 'drag-over-bottom', 'is-dragging'));
  document.querySelectorAll('.host-item').forEach(el => el.classList.remove('drag-over-top', 'drag-over-bottom', 'is-dragging'));
  setTimeout(() => {
    isDraggingHost = false;
    draggedHostId = null;
  }, 100);
}

function onFolderDragStart(e, folderId, parentId) {
  if (e.target.closest('.folder-actions') || e.target.closest('button')) {
    e.preventDefault();
    return;
  }
  e.stopPropagation();

  draggedFolderId = folderId;
  draggedFolderParentId = parentId ? String(parentId) : null;
  isDraggingFolder = true;

  e.dataTransfer.effectAllowed = 'move';
  e.dataTransfer.setData('text/rus-folder-id', folderId);
  e.dataTransfer.setData('text/plain', folderId);

  const card = document.querySelector(`.folder-card[data-folder-id="${folderId}"]`);
  if (card) {
    card.classList.add('is-dragging');
  }
}

function onFolderDragEnd(e) {
  document.querySelectorAll('.folder-card').forEach(f => {
    f.classList.remove('is-drag-over', 'drag-over-top', 'drag-over-bottom', 'is-dragging');
  });
  document.querySelectorAll('.host-item').forEach(el => {
    el.classList.remove('drag-over-top', 'drag-over-bottom', 'is-dragging');
  });
  setTimeout(() => {
    isDraggingFolder = false;
    draggedFolderId = null;
    draggedFolderParentId = null;
  }, 100);
}

function onFolderDragOver(e, folderId) {
  // Caso 1: Arrastando host para dentro da pasta
  if (draggedHostId) {
    e.preventDefault();
    e.stopPropagation();
    e.dataTransfer.dropEffect = 'move';
    const card = document.querySelector(`.folder-card[data-folder-id="${folderId}"]`);
    if (card) {
      card.classList.add('is-drag-over');
    }
    return;
  }

  // Caso 2: Arrastando pasta para reordenar
  if (draggedFolderId) {
    if (draggedFolderId === folderId) return;

    const card = document.querySelector(`.folder-card[data-folder-id="${folderId}"]`);
    if (!card) return;

    // Não permite arrastar para dentro de si mesma ou de seus próprios filhos
    if (card.closest(`.folder-card[data-folder-id="${draggedFolderId}"]`)) return;

    const targetParentId = card.dataset.parentId ? String(card.dataset.parentId) : null;

    if (targetParentId === draggedFolderParentId) {
      e.preventDefault();
      e.stopPropagation();
      e.dataTransfer.dropEffect = 'move';

      const rect = card.getBoundingClientRect();
      const midY = rect.top + rect.height / 2;

      if (e.clientY < midY) {
        card.classList.add('drag-over-top');
        card.classList.remove('drag-over-bottom');
      } else {
        card.classList.add('drag-over-bottom');
        card.classList.remove('drag-over-top');
      }
    }
  }
}

function onFolderDragLeave(e, folderId) {
  e.preventDefault();
  const card = document.querySelector(`.folder-card[data-folder-id="${folderId}"]`);
  if (card && !card.contains(e.relatedTarget)) {
    card.classList.remove('is-drag-over', 'drag-over-top', 'drag-over-bottom');
  }
}

function onFolderDrop(e, folderId) {
  e.preventDefault();
  e.stopPropagation();

  const card = document.querySelector(`.folder-card[data-folder-id="${folderId}"]`);
  const dropOnBottomHalf = card ? card.classList.contains('drag-over-bottom') : false;

  if (card) {
    card.classList.remove('is-drag-over', 'drag-over-top', 'drag-over-bottom');
  }

  // Caso 1: Soltando host dentro da pasta
  if (draggedHostId) {
    const hostId = draggedHostId;
    draggedHostId = null;
    isDraggingHost = false;

    IPC.moveHost(hostId, folderId, null);
    Toast.show('Host movido para a pasta!', 'success');
    return;
  }

  // Caso 2: Soltando pasta para reordenar
  if (draggedFolderId) {
    if (draggedFolderId === folderId) return;
    if (!card) return;

    const targetParentId = card.dataset.parentId ? String(card.dataset.parentId) : null;
    if (targetParentId === draggedFolderParentId) {
      const fromId = draggedFolderId;
      draggedFolderId = null;
      draggedFolderParentId = null;
      isDraggingFolder = false;

      reorderNodesInContainer(targetParentId, fromId, folderId, dropOnBottomHalf);
    }
  }
}

function onRootDragOver(e) {
  if (!draggedHostId) return;
  if (e.target.closest('.folder-card')) return;
  e.preventDefault();
  e.dataTransfer.dropEffect = 'move';
}

function onRootDrop(e) {
  if (!draggedHostId) return;
  if (e.target.closest('.folder-card')) return;
  e.preventDefault();
  e.stopPropagation();

  const hostId = draggedHostId;
  draggedHostId = null;
  isDraggingHost = false;

  IPC.moveHost(hostId, null, null);
  Toast.show('Host movido para a raiz!', 'success');
}

function findHostParentList(nodes, hostId) {
  if (!nodes || !hostId) return null;
  for (const n of nodes) {
    if (n.type === 'host' && (n.data.id === hostId || n.data.name === hostId)) {
      return { parentId: null, list: nodes };
    }
    if (n.type === 'folder' && n.children) {
      for (const c of n.children) {
        if (c.type === 'host' && (c.data.id === hostId || c.data.name === hostId)) {
          return { parentId: n.id, list: n.children };
        }
        if (c.type === 'folder' && c.children) {
          for (const s of c.children) {
            if (s.type === 'host' && (s.data.id === hostId || s.data.name === hostId)) {
              return { parentId: c.id, list: c.children };
            }
          }
        }
      }
    }
  }
  return null;
}

function reorderNodesInContainer(parentId, fromId, toId, dropOnBottomHalf) {
  let list = null;
  if (!parentId) {
    list = [...state.nodes];
  } else {
    const parentFolder = findFolderInNodes(state.nodes, parentId);
    if (!parentFolder || !parentFolder.children) return;
    list = [...parentFolder.children];
  }

  const fromIdx = list.findIndex(n => (n.type === 'folder' ? n.id : (n.data.id || n.data.name)) === fromId);
  const toIdx = list.findIndex(n => (n.type === 'folder' ? n.id : (n.data.id || n.data.name)) === toId);

  if (fromIdx === -1 || toIdx === -1 || fromIdx === toIdx) return;

  const [moved] = list.splice(fromIdx, 1);
  let insertIdx = toIdx;
  if (fromIdx < toIdx) {
    insertIdx = dropOnBottomHalf ? toIdx : toIdx - 1;
  } else {
    insertIdx = dropOnBottomHalf ? toIdx + 1 : toIdx;
  }
  insertIdx = Math.max(0, Math.min(insertIdx, list.length));
  list.splice(insertIdx, 0, moved);

  // Atualização otimista no estado local para resposta instantânea
  if (!parentId) {
    state.nodes = list;
  } else {
    const parentFolder = findFolderInNodes(state.nodes, parentId);
    if (parentFolder) {
      parentFolder.children = list;
    }
  }
  renderView();

  // Envia nova ordem de IDs para o backend Rust salvar em config.rtty
  const orderIds = list.map(n => n.type === 'folder' ? n.id : (n.data.id || n.data.name));
  IPC.reorderNodes(parentId, orderIds);
  Toast.show('Ordem atualizada com sucesso!', 'info');
}

function initHostDragAndDrop() {
  const hostList = document.getElementById('host-list');
  if (!hostList) return;

  // Se há busca ativa, não ativa drag & drop para evitar corromper a ordem
  if (state.hostSearchQuery && state.hostSearchQuery.trim()) {
    return;
  }

  const items = hostList.querySelectorAll('.host-item');
  items.forEach(item => {
    item.addEventListener('dragover', (e) => {
      const activeDraggedId = draggedHostId || draggedFolderId;
      if (!activeDraggedId) return;

      const targetHostId = item.dataset.hostId;
      if (targetHostId === activeDraggedId) return;

      const targetParentInfo = findHostParentList(state.nodes, targetHostId);
      if (!targetParentInfo) return;

      const draggedParentId = draggedHostId
        ? (findHostFolderId(state.nodes, draggedHostId) || null)
        : draggedFolderParentId;

      if (draggedParentId === targetParentInfo.parentId) {
        e.preventDefault();
        e.stopPropagation();
        e.dataTransfer.dropEffect = 'move';

        const rect = item.getBoundingClientRect();
        const midY = rect.top + rect.height / 2;

        if (e.clientY < midY) {
          item.classList.add('drag-over-top');
          item.classList.remove('drag-over-bottom');
        } else {
          item.classList.add('drag-over-bottom');
          item.classList.remove('drag-over-top');
        }
      }
    });

    item.addEventListener('dragleave', (e) => {
      if (!item.contains(e.relatedTarget)) {
        item.classList.remove('drag-over-top', 'drag-over-bottom');
      }
    });

    item.addEventListener('drop', (e) => {
      const activeDraggedId = draggedHostId || draggedFolderId;
      const targetHostId = item.dataset.hostId;
      const dropOnBottomHalf = item.classList.contains('drag-over-bottom');

      item.classList.remove('drag-over-top', 'drag-over-bottom');

      if (!activeDraggedId || !targetHostId || activeDraggedId === targetHostId) return;

      const targetParentInfo = findHostParentList(state.nodes, targetHostId);
      if (!targetParentInfo) return;

      const draggedParentId = draggedHostId
        ? (findHostFolderId(state.nodes, draggedHostId) || null)
        : draggedFolderParentId;

      if (draggedParentId === targetParentInfo.parentId) {
        e.preventDefault();
        e.stopPropagation();

        const fromId = activeDraggedId;
        draggedHostId = null;
        draggedFolderId = null;
        draggedFolderParentId = null;
        isDraggingHost = false;
        isDraggingFolder = false;

        reorderNodesInContainer(targetParentInfo.parentId, fromId, targetHostId, dropOnBottomHalf);
      }
    });
  });
}

// ─── Host Search & Wave Typewriter Animation Controller ──────────────────────

let searchAnimTimeout = null;
let currentSearchIconIdx = 0;
let lastPickedHostName = '';
let isSearchAnimRunning = false;

function onHostSearchInput(e) {
  state.hostSearchQuery = e.target.value;
  const placeholder = document.getElementById('host-search-placeholder');

  if (placeholder) {
    if (state.hostSearchQuery) {
      placeholder.classList.add('is-hidden');
    } else if (document.activeElement !== e.target) {
      placeholder.classList.remove('is-hidden');
    }
  }

  updateHostListFilteredView();
}

function onHostSearchFocus() {
  const placeholder = document.getElementById('host-search-placeholder');
  if (placeholder) {
    placeholder.classList.add('is-hidden');
  }
}

function onHostSearchBlur() {
  const placeholder = document.getElementById('host-search-placeholder');
  if (placeholder && !state.hostSearchQuery) {
    placeholder.classList.remove('is-hidden');
  }
}

function clearHostSearch() {
  state.hostSearchQuery = '';
  const input = document.getElementById('host-search-input');
  if (input) {
    input.value = '';
    input.focus();
  }
  const placeholder = document.getElementById('host-search-placeholder');
  if (placeholder) {
    placeholder.classList.remove('is-hidden');
  }
  updateHostListFilteredView();
}

function updateHostListFilteredView() {
  const query = (state.hostSearchQuery || '').trim().toLowerCase();
  const hostSearchWrapper = document.querySelector('.host-search-wrapper');
  if (!hostSearchWrapper) return;

  const container = document.getElementById('host-search-container');
  let clearBtn = container ? container.querySelector('.host-search-clear-btn') : null;

  if (state.hostSearchQuery) {
    if (!clearBtn && container) {
      clearBtn = document.createElement('button');
      clearBtn.type = 'button';
      clearBtn.className = 'host-search-clear-btn';
      clearBtn.title = 'Limpar busca';
      clearBtn.onclick = clearHostSearch;
      clearBtn.innerHTML = icon('x', 14);
      container.appendChild(clearBtn);
    }
  } else if (clearBtn) {
    clearBtn.remove();
  }

  const filteredHosts = query
    ? state.hosts.filter(h =>
      (h.name && h.name.toLowerCase().includes(query)) ||
      (h.address && h.address.toLowerCase().includes(query))
    )
    : state.hosts;

  let metaEl = hostSearchWrapper.querySelector('.host-search-meta');
  if (query) {
    const metaHtml = `
      <span class="host-search-count">${filteredHosts.length} de ${state.hosts.length} ${state.hosts.length === 1 ? 'host' : 'hosts'}</span>
      <button type="button" class="host-search-clear-link" onclick="clearHostSearch()">Limpar filtro</button>
    `;
    if (!metaEl) {
      metaEl = document.createElement('div');
      metaEl.className = 'host-search-meta';
      metaEl.innerHTML = metaHtml;
      hostSearchWrapper.appendChild(metaEl);
    } else {
      metaEl.innerHTML = metaHtml;
    }
  } else if (metaEl) {
    metaEl.remove();
  }

  const containerParent = hostSearchWrapper.parentNode;
  if (!containerParent) return;

  let hostListEl = document.getElementById('host-list');
  const existingEmpty = containerParent.querySelector('.empty-state');

  if (filteredHosts.length === 0 && state.hosts.length > 0) {
    if (hostListEl) hostListEl.remove();
    const emptyContent = `
      ${icon('search', 44)}
      <div class="empty-state-title">Nenhum host encontrado</div>
      <div class="empty-state-text">Nenhum servidor corresponde à busca "<strong>${escHtml(state.hostSearchQuery)}</strong>".</div>
      <button type="button" class="btn btn--secondary mt-4" onclick="clearHostSearch()">
        ${icon('undo', 16)} Limpar Busca
      </button>
    `;
    if (!existingEmpty) {
      const emptyDiv = document.createElement('div');
      emptyDiv.className = 'empty-state animate-fade-in';
      emptyDiv.innerHTML = emptyContent;
      containerParent.appendChild(emptyDiv);
    } else {
      existingEmpty.innerHTML = emptyContent;
    }
  } else {
    if (existingEmpty) existingEmpty.remove();
    const cardsHtml = query
      ? filteredHosts.map(h => hostCard(h, true, h.folder_path)).join('')
      : renderNodeTree(state.nodes);

    if (hostListEl) {
      if (!query) {
        hostListEl.setAttribute('ondragover', 'onRootDragOver(event)');
        hostListEl.setAttribute('ondrop', 'onRootDrop(event)');
      } else {
        hostListEl.removeAttribute('ondragover');
        hostListEl.removeAttribute('ondrop');
      }
      hostListEl.innerHTML = cardsHtml;
    } else {
      const listDiv = document.createElement('div');
      listDiv.className = 'host-list';
      listDiv.id = 'host-list';
      if (!query) {
        listDiv.setAttribute('ondragover', 'onRootDragOver(event)');
        listDiv.setAttribute('ondrop', 'onRootDrop(event)');
      }
      listDiv.innerHTML = cardsHtml;
      containerParent.appendChild(listDiv);
    }
    if (!query) {
      initHostDragAndDrop();
    }
  }
}

function cycleSearchIcon() {
  const box = document.getElementById('host-search-icon-box');
  if (!box) return;
  const icons = box.querySelectorAll('.host-search-icon');
  if (icons.length === 0) return;

  icons.forEach(el => el.classList.remove('active'));
  currentSearchIconIdx = (currentSearchIconIdx + 1) % icons.length;
  icons[currentSearchIconIdx].classList.add('active');
}

function getNextSearchPlaceholderName() {
  const availableNames = state.hosts
    .map(h => (h.name || '').trim())
    .filter(name => name.length > 0);

  if (availableNames.length === 0) {
    const defaults = ['Servidor Produção', 'Ubuntu VPS', 'Database Cluster', 'Gateway VPN', 'Kubernetes Node'];
    const filtered = defaults.filter(n => n !== lastPickedHostName);
    const pick = filtered[Math.floor(Math.random() * filtered.length)] || defaults[0];
    lastPickedHostName = pick;
    return pick;
  }

  if (availableNames.length === 1) {
    lastPickedHostName = availableNames[0];
    return availableNames[0];
  }

  const candidates = availableNames.filter(n => n !== lastPickedHostName);
  const pool = candidates.length > 0 ? candidates : availableNames;
  const chosen = pool[Math.floor(Math.random() * pool.length)];
  lastPickedHostName = chosen;
  return chosen;
}

function startHostSearchAnimation() {
  stopHostSearchAnimation();

  const waveTextEl = document.getElementById('host-search-wave-text');
  if (!waveTextEl) return;

  isSearchAnimRunning = true;
  runTypewriterLoop();
}

function stopHostSearchAnimation() {
  isSearchAnimRunning = false;
  if (searchAnimTimeout) {
    clearTimeout(searchAnimTimeout);
    searchAnimTimeout = null;
  }
}

function runTypewriterLoop() {
  if (!isSearchAnimRunning) return;

  const waveTextEl = document.getElementById('host-search-wave-text');
  if (!waveTextEl) {
    stopHostSearchAnimation();
    return;
  }

  const targetName = getNextSearchPlaceholderName();
  let charIdx = 0;

  function typeNextChar() {
    if (!isSearchAnimRunning) return;
    const currentEl = document.getElementById('host-search-wave-text');
    if (!currentEl) { stopHostSearchAnimation(); return; }

    if (charIdx < targetName.length) {
      const char = targetName[charIdx];
      const span = document.createElement('span');
      span.className = 'wave-char';
      span.style.setProperty('--char-idx', String(charIdx));
      span.textContent = char === ' ' ? '\u00A0' : char;
      currentEl.appendChild(span);
      charIdx++;

      const speed = 70 + Math.floor(Math.random() * 45);
      searchAnimTimeout = setTimeout(typeNextChar, speed);
    } else {
      searchAnimTimeout = setTimeout(startDeleting, 2200);
    }
  }

  function startDeleting() {
    if (!isSearchAnimRunning) return;
    deleteNextChar();
  }

  function deleteNextChar() {
    if (!isSearchAnimRunning) return;
    const currentEl = document.getElementById('host-search-wave-text');
    if (!currentEl) { stopHostSearchAnimation(); return; }

    if (currentEl.lastChild) {
      currentEl.removeChild(currentEl.lastChild);
      searchAnimTimeout = setTimeout(deleteNextChar, 42);
    } else {
      cycleSearchIcon();
      searchAnimTimeout = setTimeout(() => {
        runTypewriterLoop();
      }, 420);
    }
  }

  typeNextChar();
}

//  Utility Functions 

function escHtml(str) {
  if (typeof str !== 'string') return '';
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

function escAttr(str) {
  if (typeof str !== 'string') return '';
  return str.replace(/&/g, '&amp;').replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

//  Global Event Listeners 

document.addEventListener('DOMContentLoaded', () => {
  // Inicia conexão WebSocket em tempo real para live updates
  connectWebSocket();

  // Close context menu and host icon menu on click outside
  document.addEventListener('click', (e) => {
    if (!e.target.closest('.context-menu')) closeContextMenu();
    if (!e.target.closest('#host-icon-picker-container')) closeHostIconMenu();
  });

  // Close modal on backdrop click
  document.getElementById('modal-overlay').addEventListener('click', (e) => {
    if (e.target === e.currentTarget) hideModal();
  });

  // ESC key handlers
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      hideModal();
      closeContextMenu();
      closeHostIconMenu();
    }
  });

  // Bloqueio do menu de contexto padrão do navegador (exceto nos itens que possuem menu de contexto próprio)
  document.addEventListener('contextmenu', (e) => {
    if (!e.target.closest('#context-menu') && !e.target.closest('.host-card') && !e.target.closest('.host-item') && !e.target.closest('.bridge-item')) {
      e.preventDefault();
    }
  });

  // Bloqueio de teclas de sistema de navegador (F12, F7, atalhos de DevTools, reload, etc.)
  window.addEventListener('keydown', (e) => {
    // F12: DevTools
    if (e.key === 'F12' || e.keyCode === 123) {
      e.preventDefault();
      e.stopPropagation();
      return false;
    }
    // F7: Caret browsing / acessibilidade de navegação
    if (e.key === 'F7' || e.keyCode === 118) {
      e.preventDefault();
      e.stopPropagation();
      return false;
    }
    // F5 / Ctrl+R: Recarregar
    if (e.key === 'F5' || (e.ctrlKey && (e.key === 'r' || e.key === 'R'))) {
      e.preventDefault();
      e.stopPropagation();
      return false;
    }
    // Ctrl+Shift+I / J / C: Atalhos de DevTools
    if (e.ctrlKey && e.shiftKey && ['I', 'i', 'J', 'j', 'C', 'c'].includes(e.key)) {
      e.preventDefault();
      e.stopPropagation();
      return false;
    }
    // Ctrl+U / Ctrl+S / Ctrl+P: Exibir código-fonte, Salvar, Imprimir
    if (e.ctrlKey && ['u', 'U', 's', 'S', 'p', 'P'].includes(e.key)) {
      e.preventDefault();
      e.stopPropagation();
      return false;
    }
  }, true);

  // Developer link
  document.getElementById('dev-link').addEventListener('click', () => {
    IPC.openUrl('https://byvitor.com.br/');
  });

  // Render inicial (sem dados ainda — IPC preenche depois)
  renderSidebar();
  renderView();

  // Solicita dados iniciais ao backend (uma única vez cada)
  IPC.requestConfig();
  IPC.requestClientConfig();

});


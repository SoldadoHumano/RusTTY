// RusTTY Webview  SPA Application (Vanilla JS)
// Complete client-side application: routing, state, IPC, views, components

'use strict';

//  Lucide SVG Icon Paths (embedded, no CDN) 
const ICON_PATHS = {
  home: '<path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/><path d="M3 10a2 2 0 0 1 .709-1.528l7-5.999a2 2 0 0 1 2.582 0l7 5.999A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
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
  save: '<path d="M15.2 3a2 2 0 0 1 1.4.6l3.8 3.8a2 2 0 0 1 .6 1.4V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z"/><path d="M17 21v-7a1 1 0 0 0-1-1H8a1 1 0 0 0-1 1v7"/><path d="M7 3v4a1 1 0 0 0 1 1h7"/>',
  x: '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>',
  undo: '<path d="M3 7v6h6"/><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"/>',
  trash: '<path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/>',
  edit: '<path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/><path d="m15 5 4 4"/>',
  'alert-triangle': '<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/><path d="M12 9v4"/><path d="M12 17h.01"/>',
  check: '<path d="M20 6 9 17l-5-5"/>',
  'globe-lock': '<path d="M15.686 15A14.5 14.5 0 0 1 12 22a14.5 14.5 0 0 1 0-20 10 10 0 1 0 9.542 13"/><path d="M2 12h8.5"/><path d="M20 6V4a2 2 0 1 0-4 0v2"/><rect width="8" height="5" x="14" y="6" rx="1"/>',
  folder: '<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>',
  'heart-plus': '<path d="M13.5 2.764a4.97 4.97 0 0 0-2.83 1.3l-.67.66-.67-.66a5 5 0 0 0-7.08 7.07L12 20.84l3.7-3.71"/><path d="M16 14v6"/><path d="M19 17h-6"/>',
  'at-sign': '<circle cx="12" cy="12" r="4"/><path d="M16 8v5a3 3 0 0 0 6 0v-1a10 10 0 1 0-4 8"/>',
  globe: '<circle cx="12" cy="12" r="10"/><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"/><path d="M2 12h20"/>',
  plus: '<path d="M5 12h14"/><path d="M12 5v14"/>',
  'chevron-right': '<path d="m9 18 6-6-6-6"/>',
  search: '<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>',
  info: '<circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>',
  'arrow-up-right': '<path d="M7 7h10v10"/><path d="M7 17 17 7"/>',
};

/** Creates an SVG icon element */
function icon(name, size = 20) {
  const paths = ICON_PATHS[name];
  if (!paths) return '';
  return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${paths}</svg>`;
}

//  IPC Layer 

const IPC = {
  send(message) {
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(JSON.stringify(message));
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

  saveHost(data, editIndex) { this.send({ type: 'save_host', data, edit_index: editIndex ?? null }); },
  deleteHost(index) { this.send({ type: 'delete_host', index }); },
  saveBridge(data, editIndex) { this.send({ type: 'save_bridge', data, edit_index: editIndex ?? null }); },
  deleteBridge(index) { this.send({ type: 'delete_bridge', index }); },

  saveSetting(key, value) { this.send({ type: 'save_setting', key, value }); },
  saveCustomization(data) { this.send({ type: 'save_customization', data }); },
  deleteKeyword(index) { this.send({ type: 'delete_keyword', index }); },

  openUrl(url) { this.send({ type: 'open_url', url }); },
};

// Rust �  Frontend callback
window.__rustCallback = function (dataStr) {
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

  // Data from Rust
  hosts: [],
  bridges: [],
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
    selectedBridge: null, legacySsh: false, showPassword: false, error: null
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
      state.hosts = msg.hosts || [];
      state.bridges = msg.bridges || [];

      // Se há uma navegação pendente (ex: após salvar host/bridge), executa agora
      // com os dados já atualizados — evita o render duplo com dados antigos.
      if (state.pendingNavigateAfterConfig) {
        const dest = state.pendingNavigateAfterConfig;
        state.pendingNavigateAfterConfig = null;
        navigate(dest);
      } else {
        if (state.currentView === 'home') renderView();
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
      if (state.currentView === 'settings') renderView();
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
      Toast.show(msg.message, 'success', 8000);
      break;
  }
}

//  Navigation / Router 

function navigate(view, params = {}) {
  // Reset context menu
  closeContextMenu();

  state.currentView = view;

  // Prepare forms when entering form views
  if (view === 'new-host' && params.editIndex != null) {
    state.editingHostIndex = params.editIndex;
    const host = state.hosts[params.editIndex];
    if (host) {
      state.hostForm = {
        name: host.name, address: host.address, port: String(host.port),
        username: host.username, password: '', allowDomain: host.allow_domain || false,
        enableIcmp: host.enable_icmp ?? true, enableBridge: !!host.bridge_id,
        selectedBridge: host.bridge_id || null, legacySsh: host.legacy_ssh || false,
        showPassword: false, error: null,
      };
    }
  } else if (view === 'new-host') {
    state.editingHostIndex = null;
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

function viewHome() {
  const hostCards = state.hosts.length === 0
    ? `<div class="empty-state">
         ${icon('server', 48)}
         <div class="empty-state-title">Nenhum host cadastrado</div>
         <div class="empty-state-text">Clique em "Novo Host" para adicionar seu primeiro servidor.</div>
       </div>`
    : `<div class="host-list">
         ${state.hosts.map((h, i) => hostCard(h, i)).join('')}
       </div>`;

  return `
    <div class="view-inner--wide">
      <div class="page-header">
        <div class="page-header-left">
          <span class="page-title-icon">${icon('server', 28)}</span>
          <div>
            <h1 class="page-title">Suas Conexões</h1>
            <p class="page-subtitle">Gerencie seus servidores e instâncias.</p>
          </div>
        </div>
        <div class="page-header-actions">
          <button type="button" class="btn btn--secondary" onclick="navigate('quick-connect')">
            ${icon('plug')} Conexão Rápida
          </button>
          <button type="button" class="btn btn--primary" onclick="navigate('new-host')">
            ${icon('plus')} Novo Host
          </button>
        </div>
      </div>
      ${hostCards}
    </div>
  `;
}

function hostCard(host, index) {
  const icmpState = state.icmpStatus[String(index)];
  let iconClass = '';
  if (host.enable_icmp && state.clientConfig.global_icmp) {
    if (icmpState === true) iconClass = 'host-icon-wrapper--online';
    else if (icmpState === false) iconClass = 'host-icon-wrapper--offline';
  }

  const bridgeTag = host.bridge_id
    ? `<span class="bridge-indicator">${icon('network', 12)} Ponte</span>` : '';

  return `
    <div class="host-item stagger-item" data-host-index="${index}"
         onclick="IPC.openTerminal('${escAttr(host.name)}')"
         oncontextmenu="showHostContextMenu(event, ${index})">
      <div class="host-icon-wrapper ${iconClass}">
        ${icon('terminal', 20)}
      </div>
      <div class="host-info">
        <div class="host-name">${escHtml(host.name)}</div>
        <div class="host-detail">${escHtml(host.username)}@${escHtml(host.address)}:${host.port}</div>
      </div>
      <div class="host-meta">
        ${bridgeTag}
        <span class="host-badge">:${host.port}</span>
      </div>
      <div class="host-actions">
        <button type="button" class="btn btn--icon" onclick="event.stopPropagation(); navigate('new-host', { editIndex: ${index} })" title="Editar">
          ${icon('edit', 16)}
        </button>
        <button type="button" class="btn btn--icon btn--icon-danger" onclick="event.stopPropagation(); confirmDeleteHost(${index})" title="Excluir">
          ${icon('trash', 16)}
        </button>
      </div>
    </div>
  `;
}

function updateIcmpIndicators() {
  document.querySelectorAll('.host-item').forEach(el => {
    const idx = el.dataset.hostIndex;
    const host = state.hosts[idx];
    if (!host || !host.enable_icmp || !state.clientConfig.global_icmp) return;
    const wrapper = el.querySelector('.host-icon-wrapper');
    if (!wrapper) return;
    const icmpState = state.icmpStatus[String(idx)];
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
        <div class="input-group">
          <label class="input-label">${icon('monitor')} Apelido / Nome</label>
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
  if (!port || port < 1 || port > 65535) { f.error = 'Porta inválida (1�65535).'; renderView(); return; }

  IPC.saveHost({
    name: f.name.trim(), address: f.address.trim(), port,
    username: f.username.trim(), password: f.password,
    allow_domain: f.allowDomain, enable_icmp: f.enableBridge ? false : f.enableIcmp,
    bridge_id: f.enableBridge ? f.selectedBridge : null,
    legacy_ssh: f.legacySsh,
  }, state.editingHostIndex);
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
    <div class="view-inner--wide">
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
      <div class="host-meta">
        <span class="host-badge">:${bridge.port}</span>
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
      const webviewBadge = s.webview_only ? '<span class="badge badge--sm badge--warning ml-2" style="font-size:0.6rem; padding: 2px 4px; vertical-align: middle;">WEBVIEW EXCLUSIVO</span>' : '';
      const labelHtml = s.label + webviewBadge;

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
          ${aboutRow('Nome do Cliente', 'RusTTY Beta')}
          ${aboutRow('Versão do Cliente', 'Beta v1.0.0')}
          ${aboutRow('Data da Versão', '28/08/2026')}
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

      <div class="settings-group-title">PADR�"ES DE SISTEMA</div>
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
    <div class="view-inner--wide">
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
    <label class="checkbox-wrapper" onclick="toggleCheckbox('${bindKey}')">
      <input type="checkbox" ${checked ? 'checked' : ''}>
      <span class="checkbox-box">
        ${icon('check', 14)}
      </span>
      <span class="checkbox-label">${label}</span>
    </label>
  `;
}

function toggleSwitch(settingKey, checked) {
  return `
    <label class="toggle-switch">
      <input type="checkbox" ${checked ? 'checked' : ''} onchange="toggleSetting('${settingKey}')">
      <span class="toggle-track"></span>
      <span class="toggle-knob"></span>
    </label>
  `;
}

function toggleSetting(key) {
  const newVal = !state.clientConfig[key];
  state.clientConfig[key] = newVal;
  IPC.saveSetting(key, newVal);

  // If customization was toggled, update the sidebar without re-rendering the whole view
  if (key === 'enable_customization') {
    renderSidebar();
  }
}

function toggleCheckbox(bindKey) {
  const parts = bindKey.split('.');
  let obj = state;
  for (let i = 0; i < parts.length - 1; i++) obj = obj[parts[i]];
  obj[parts[parts.length - 1]] = !obj[parts[parts.length - 1]];

  // Quando "Habilitar ponte" é ativado sem bridge já selecionada, inicializa
  // com a primeira bridge disponível. Sem isso, o <select> mostraria a primeira
  // opção visualmente, mas state.hostForm.selectedBridge ficaria null e o
  // host seria salvo sem bridge_id se o usuário não interagisse com o <select>.
  if (bindKey === 'hostForm.enableBridge' && state.hostForm.enableBridge && !state.hostForm.selectedBridge) {
    state.hostForm.selectedBridge = state.bridges[0]?.id ?? null;
  }

  renderView();
}

// Modals 

function showModal(html) {
  const overlay = document.getElementById('modal-overlay');
  const content = document.getElementById('modal-content');
  content.innerHTML = html;
  overlay.classList.add('visible');
}

function hideModal() {
  document.getElementById('modal-overlay').classList.remove('visible');
}

function confirmDeleteHost(index) {
  const host = state.hosts[index];
  if (!host) return;
  showModal(`
    <div class="modal-header">
      <div class="modal-title modal-title--danger">${icon('alert-triangle', 22)} Deletar "${escHtml(host.name)}"?</div>
      <button type="button" class="btn btn--icon" onclick="hideModal()">${icon('x', 18)}</button>
    </div>
    <div class="modal-body">
      <p>Esta ação é permanente e irreversível.</p>
    </div>
    <div class="modal-actions">
      <button type="button" class="btn btn--ghost" onclick="hideModal()">Cancelar</button>
      <button type="button" class="btn btn--danger" onclick="hideModal(); IPC.deleteHost(${index})">
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
  show(message, type = 'info', duration = 4000) {
    const container = document.getElementById('toast-container');
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

function showHostContextMenu(e, index) {
  e.preventDefault();
  e.stopPropagation();
  const menu = document.getElementById('context-menu');
  menu.innerHTML = `
    <button type="button" class="context-menu-item" onclick="closeContextMenu(); navigate('new-host', { editIndex: ${index} })">
      ${icon('edit', 16)} Editar host
    </button>
    <div class="context-menu-separator"></div>
    <button type="button" class="context-menu-item context-menu-item--danger" onclick="closeContextMenu(); confirmDeleteHost(${index})">
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
  // Close context menu on click outside
  document.addEventListener('click', (e) => {
    if (!e.target.closest('.context-menu')) closeContextMenu();
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
    }
  });

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


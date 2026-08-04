# Visão Geral do RusTTY

O **RusTTY** é um cliente SSH e emulador de terminal escrito em Rust, focado em performance nativa, baixo consumo de recursos e segurança no armazenamento de credenciais. O projeto surgiu de uma insatisfação pessoal com as opções disponíveis no Windows — ou eram lentas demais, ou tinham interface terrível, ou armazenavam senhas de forma que deixava a desejar.

---

## Por que Rust, por que esse design

A maioria dos clientes SSH para Windows tem um dos seguintes problemas:

- **Overhead desnecessário**: soluções baseadas em Electron ou tecnologias web empacotam um runtime inteiro de Node.js + motor de renderização HTML para exibir o que é essencialmente texto em uma grade. O custo de memória e CPU é desproporcional à tarefa.
- **Sem isolamento de sessão**: clientes que concentram tudo em um único processo compartilham estado. Se uma sessão trava ou o servidor do outro lado fica maluco, o gerenciador inteiro pode ser afetado.
- **Credenciais armazenadas de forma ingênua**: texto claro, ou criptografia simétrica com chave fixa embutida no binário — o que é equivalente a texto claro para quem sabe onde procurar.

O RusTTY tentou resolver cada um desses pontos. Rust elimina o overhead de runtime e GC. O modelo de processos independentes isola sessões. A DPAPI + AES-256-GCM cuida das credenciais.

---

## Como funciona a arquitetura de janelas

Quando você conecta a um host, o gerenciador principal spawna um **processo filho independente** que hospeda o emulador de terminal e a sessão SSH. Cada conexão é um processo separado do sistema operacional.

O que isso significa na prática:

- Uma sessão travando não afeta o gerenciador nem as outras sessões abertas.
- Você pode mover janelas de terminal para monitores diferentes livremente — elas são janelas de SO normais.
- Quando você fecha a janela, o processo morre e todos os recursos (memória, descritores de arquivo, conexão SSH) são liberados imediatamente pelo SO.

---

## Capacidades

### Gerenciador de conexões
Lista de hosts persistida localmente, com suporte a SSH via Jump Host, monitoramento de disponibilidade por ICMP e autenticação por senha ou chave privada.

### Emulação de terminal
O emulador suporta `xterm-256color` completo — cores de 256 e truecolor (RGB 24-bit), sequências de escape modernas, redimensionamento dinâmico com notificação via `window-change` ao servidor, e scrollback configurável.

### Personalização de renderização
Destaque de palavras-chave em tempo real e colorização automática de endereços IPv4/IPv6 com suporte a distinguir IPs públicos de privados.

---

O código está disponível para leitura e contribuição. Se algo não funciona como esperado, issues e pull requests são bem-vindos.
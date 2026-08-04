# Operação do Terminal SSH

Depois de conectar, o RusTTY abre um processo filho com o emulador de terminal. Esta seção cobre o que acontece nessa janela e como tirar proveito das funcionalidades disponíveis.

---

## Como o I/O funciona

O emulador roda um loop assíncrono (`tokio::select!`) que processa dois fluxos ao mesmo tempo:

1. **Dados do servidor → emulador**: bytes recebidos pelo canal SSH são parseados pelo interpretador de sequências de escape e renderizados na grade de células do terminal.
2. **Input do teclado → servidor**: teclas pressionadas são codificadas e enviadas pelo canal SSH ao processo remoto.

O tipo de terminal negociado com o servidor é `xterm-256color`, configurado durante a alocação do PTY. Isso é o que a variável `$TERM` vai conter no lado remoto.

---

## Copiar e colar

O conflito clássico de `Ctrl+C` (SIGINT no Unix) vs. copiar/colar é resolvido com atalhos alternativos:

| Ação | Atalho |
|---|---|
| Copiar seleção | `Ctrl + Shift + C` |
| Colar | `Ctrl + Shift + V` |
| Copiar (se há seleção) / Colar (sem seleção) | Botão direito do mouse |

Selecione texto arrastando o mouse. O clique direito é o atalho mais rápido no dia a dia.

---

## Command Palette

A Command Palette é acessada com `Ctrl + <tecla configurada>` (padrão: `Ctrl + .`) e oferece operações sobre o buffer do terminal:

**Copy All** — copia o conteúdo integral do scrollback, incluindo o histórico que já saiu da área visível. Útil pra capturar saídas longas de diagnóstico sem ter que selecionar manualmente.

**Copy Last N Lines** — copia as últimas N linhas. Informe o número no campo e confirme. Prático pra pegar só o trecho relevante de um log longo.

**Clear Terminal** — apaga a área visível e o histórico de scrollback. Não afeta o processo remoto — é só limpeza visual local.

---

## Scrollback

O buffer de scrollback retém as linhas que saíram da tela. Navegue com a roda do mouse. A quantidade de linhas por evento de scroll e o limite total do buffer são configuráveis em Configurações.

---

## Redimensionamento

Quando você redimensiona a janela do terminal, o emulador recalcula colunas e linhas e envia um `window-change` ao servidor. O kernel remoto manda `SIGWINCH` para o processo em foreground, que pode reajustar o layout — `vim`, `tmux`, `htop` e similares respondem a isso automaticamente.

---

## Tamanho de fonte

Configurável em Configurações. Alterar o tamanho implica recalcular as dimensões do terminal em colunas e linhas, o que dispara automaticamente o `window-change` descrito acima.

---

## Fechar a sessão

Fechar a janela envia `SSH_MSG_CHANNEL_EOF` + `SSH_MSG_CHANNEL_CLOSE` ao servidor, que encerra a sessão e libera os recursos alocados do lado de lá. Você também pode simplesmente digitar `exit` ou `logout` no shell remoto — o servidor manda um `exit-status` e o terminal fecha sozinho.

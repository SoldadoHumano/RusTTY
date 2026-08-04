# Diagnóstico e Resolução de Problemas

Esta seção cobre os erros mais comuns, as causas prováveis e como investigar cada um.

---

## Falhas de conexão

### Connection refused

O RusTTY retorna erro imediatamente, sem aguardar timeout. Isso significa que o host chegou a responder — só que recusou a conexão.

Causas mais comuns:

| Causa | Como verificar |
|---|---|
| Daemon SSH não está rodando | `sudo systemctl status sshd` no servidor |
| SSH configurado em porta diferente de 22 | Checar `Port` no `/etc/ssh/sshd_config` |
| Firewall bloqueando a porta | `sudo iptables -L INPUT -n` ou `sudo nft list ruleset` |
| Porta ou endereço errado no RusTTY | Revisar o cadastro do host |

Para testar alcançabilidade do lado do cliente:

```powershell
Test-NetConnection -ComputerName <IP> -Port 22
```

---

### Timeout (sem resposta)

O RusTTY espera e não recebe resposta. Diferente do `connection refused`, aqui os pacotes estão sendo descartados silenciosamente.

Causas prováveis: o host não é roteável a partir da sua máquina (sub-rede privada sem rota, VPN desconectada), ou um firewall de borda está descartando SYN sem responder RST.

```powershell
tracert <IP_DESTINO>
route print
```

---

### Authentication failed

A conexão TCP e o handshake SSH funcionaram, mas a autenticação foi recusada.

**Para senha:**
- Credencial incorreta salva no RusTTY.
- Usuário inexistente no servidor.
- `PasswordAuthentication no` no `sshd_config` do servidor.
- Conta bloqueada ou expirada.

**Para chave privada:**
- Chave pública não está no `~/.ssh/authorized_keys` do servidor.
- Permissões incorretas em `~/.ssh/` ou `authorized_keys` no servidor (veja a seção **Autenticação por Chave**).
- `PubkeyAuthentication no` no `sshd_config`.
- Passphrase errada para a chave.

Para ver o que o servidor está registrando:

```bash
sudo journalctl -u sshd -n 50 --no-pager
# ou
sudo tail -50 /var/log/auth.log
```

---

## Falhas com Jump Host

### "Ponte falhou ao rotear para \<destino\>"

O RusTTY autenticou no Jump Host com sucesso mas não conseguiu abrir o canal `direct-tcpip`.

Causa mais provável: `AllowTcpForwarding no` no `sshd_config` do Jump Host.

```bash
# No Jump Host
sudo sshd -T | grep allowtcpforwarding
```

Se retornar `no`, altere para `yes` em `/etc/ssh/sshd_config` e recarregue:

```bash
sudo systemctl reload sshd
```

Causa alternativa: o Jump Host não tem conectividade TCP de saída para o destino.

```bash
nc -zv <IP_DESTINO> <PORTA_DESTINO>
```

---

### "Falha ao conectar via ponte em \<destino\>"

O canal `direct-tcpip` foi aberto, mas o segundo handshake SSH (com o destino) falhou.

Causas possíveis:
- O daemon SSH no destino está recusando conexões vindas do IP do Jump Host (regra de firewall no destino).
- O destino não está acessível a partir do Jump Host apesar do canal ter sido aberto.

---

## Problemas de renderização

### Caracteres corrompidos / encoding errado

Acentos e caracteres especiais aparecem como símbolos estranhos. Quase sempre é problema de locale no servidor:

```bash
locale  # verificar encoding atual

export LANG=pt_BR.UTF-8
export LC_ALL=pt_BR.UTF-8

echo 'export LANG=pt_BR.UTF-8' >> ~/.bashrc
```

---

### Layout quebrado em vim / htop / tmux

A interface de aplicações ncurses fica desalinhada ou sobreposta. Geralmente é a variável `$TERM` incorreta no servidor:

```bash
echo $TERM  # deve ser xterm-256color
stty size   # deve bater com as dimensões visíveis

export TERM=xterm-256color
```

---

## Performance

Se o consumo de CPU ou bateria estiver alto:

1. Ative o **Modo de Performance** nas Configurações.
2. Reduza `max_scrollback_lines` se estiver muito alto.
3. Desative ICMP para hosts que não precisam de monitoramento contínuo.

---

## Coletando informações para reportar um bug

Se você for abrir uma issue, inclua:

- Versão do RusTTY (visível em Configurações ou no "Sobre").
- Versão do Windows e build.
- Versão do daemon SSH no servidor: `sshd -V`.
- Método de autenticação usado (senha ou chave).
- Se Jump Host está envolvido.
- Mensagem de erro exata retornada pelo RusTTY.

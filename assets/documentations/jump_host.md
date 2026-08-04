# SSH via Ponte (Jump Host / Bastion Host)

Em redes segmentadas, é comum ter servidores de destino que não são acessíveis diretamente — ficam em sub-redes privadas sem rota pra internet, acessíveis só a partir de um host específico na borda da rede. O RusTTY suporta esse padrão nativamente, sem precisar de `ssh -J` externo ou configurar `ProxyCommand`.

---

## Como funciona

O fluxo envolve dois handshakes SSH encadeados:

```
[RusTTY] ──SSH──► [Jump Host] ──direct-tcpip──► [Destino]
          Estágio 1             Estágio 2
```

**Estágio 1** — conexão TCP normal ao Jump Host, handshake SSH completo, autenticação com as credenciais do Jump Host.

**Estágio 2** — o RusTTY pede ao daemon SSH do Jump Host pra abrir um canal `direct-tcpip` (RFC 4254 §7.2). Esse canal instrui o servidor SSH a fazer uma conexão TCP de saída para o endereço e porta do destino e encaminhar os dados bidirecionalmente. O Jump Host não inspeciona o conteúdo — ele só encaminha bytes.

Com o canal aberto, o RusTTY executa um segundo handshake SSH completo através desse pipe, desta vez com o host de destino. O destino vê a conexão como vindo do IP do Jump Host, não do cliente original.

---

## Pré-requisito no Jump Host: `AllowTcpForwarding`

Para que o canal `direct-tcpip` funcione, o daemon SSH do Jump Host precisa ter TCP forwarding habilitado. Isso é controlado pela diretiva `AllowTcpForwarding` no `sshd_config`:

```
AllowTcpForwarding yes
```

Se a diretiva não existir no arquivo, o comportamento padrão do OpenSSH é `yes`. Se estiver explicitamente como `no` ou `local`, o servidor vai recusar o canal e o RusTTY vai retornar erro no Estágio 2.

Para verificar sem editar o arquivo:

```bash
sudo sshd -T | grep allowtcpforwarding
```

Para recarregar o daemon após qualquer mudança:

```bash
# systemd
sudo systemctl reload sshd

# OpenRC
sudo rc-service sshd reload
```

---

## Conectividade do Jump Host ao destino

Além da configuração do daemon, o Jump Host precisa ter alcançabilidade TCP de saída para o endereço e porta do destino. Se houver regras de firewall bloqueando esse tráfego de saída, o canal abre mas a conexão com o destino falha.

Para testar a partir do Jump Host:

```bash
nc -zv <IP_DESTINO> <PORTA_DESTINO>

# Ou com timeout explícito
timeout 5 bash -c "echo > /dev/tcp/<IP_DESTINO>/<PORTA_DESTINO>" && echo "OK" || echo "Sem acesso"
```

---

## Configuração no RusTTY

No formulário de cadastro do host de destino:

1. Habilite **"Ponte SSH"**.
2. Preencha os campos do Jump Host: endereço, porta (padrão `22`), usuário e credencial.
3. Os demais campos do formulário referem-se ao host de destino final.

As credenciais do Jump Host e do destino são completamente independentes — você pode usar senha em um e chave privada no outro.

---

## Algumas observações

**Logs de auditoria no destino**: do ponto de vista do host de destino, as conexões chegam com o IP do Jump Host como origem. Se o ambiente tem logs de autenticação, eles vão registrar o Jump Host, não a sua estação. Leve isso em conta se você precisa rastrear acessos por usuário.

**Segurança do usuário de ponte**: o usuário configurado no Jump Host não precisa de muito — só a capacidade de abrir um canal `direct-tcpip`. Em ambientes mais restritivos, vale usar `ForceCommand /bin/false` para esse usuário, impedindo sessões interativas mas mantendo o forwarding funcionando.

**Dupla criptografia**: o tráfego entre o RusTTY e o Jump Host é cifrado pelo SSH do Estágio 1. O tráfego SSH do Estágio 2 é cifrado pela sua própria camada. Na prática, o trecho RusTTY → Jump Host passa por duas camadas de criptografia.

---

## Limitações atuais

| Limitação | Detalhe |
|---|---|
| Múltiplos saltos | Só um Jump Host intermediário é suportado. Topologias com dois ou mais saltos não são suportadas nativamente. |
| Host key verification | Não é feita em nenhum dos estágios. Veja a seção **Segurança** para os detalhes. |
| SOCKS proxy | O mecanismo de ponte é exclusivo para sessões de terminal. Dynamic port forwarding (SOCKS) não é suportado. |

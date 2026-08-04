# Gerenciamento de Hosts e Conexões

No RusTTY, um **Host** é uma entrada no inventário de conexões — endereço, porta, credenciais e preferências, tudo persistido localmente e criptografado.

Há duas formas de conectar: pelo **inventário** (host cadastrado, credenciais salvas) ou pela **Conexão Rápida** (acesso pontual sem salvar nada).

---

## Cadastrar um novo host

Clique em **"Novo Host"** para abrir o formulário:

### Nome

Rótulo de exibição local, não tem impacto no protocolo. Uma convenção como `[prod] db-primary` ou `[dev] api-01` ajuda bastante quando o inventário cresce.

### Endereço e Porta

Aceita IPv4, IPv6 e FQDNs. Por padrão, o campo valida só IPs numéricos — habilite **"Permitir Domínios"** se precisar de hostname. Porta padrão: `22`.

### Autenticação

**Usuário e Senha** para autenticação por senha. A senha é criptografada ao salvar e não aparece em texto claro depois — há um ícone de olho para visualização temporária.

Para autenticação por chave privada, veja a seção **Autenticação por Chave**.

### ICMP

Com essa opção habilitada, o host entra na rotina de ping periódico e exibe um indicador de status na listagem principal. Veja as considerações de escala em **Boas Práticas**.

### Ponte SSH

Roteia a conexão através de um Jump Host intermediário. Veja a seção **SSH via Ponte** para funcionamento e pré-requisitos.

---

## Gerenciar Pontes SSH

O RusTTY mantém um cadastro separado de **pontes SSH** — servidores de salto reutilizáveis entre vários hosts. Acesse pelo menu **"Pontes"** na barra lateral.

No cadastro da ponte você define: nome, endereço, porta e credenciais do servidor de salto. Uma vez cadastrada, a ponte fica disponível para seleção no formulário de qualquer host.

---

## Conexão Rápida

A aba **Conexão Rápida** permite conectar sem salvar nada em disco. Os dados existem apenas em memória durante a sessão e somem ao fechar o terminal.

Protocolos disponíveis:

| Protocolo | Uso |
|---|---|
| SSH | Sessão SSH padrão |
| Telnet | Conexão Telnet simples |
| Serial | Comunicação por porta serial |

Útil para acesso pontual a equipamentos de terceiros, dispositivos de rede ou qualquer host que você não quer manter no inventário.

---

## Monitoramento ICMP

Para hosts com ICMP habilitado, o RusTTY dispara pings periódicos em background:

| Indicador | Significado |
|---|---|
| Verde | Respondeu dentro do timeout |
| Vermelho | Sem resposta |

ICMP pode estar filtrado no firewall sem que o SSH esteja inacessível. Um indicador vermelho não implica necessariamente que a conexão vai falhar — só que o host não responde a ping.

---

## Editar e remover hosts

O ícone de lápis abre o formulário preenchido. Você pode substituir a senha sem informar o valor atual.

O ícone de lixeira pede confirmação antes de excluir. A operação remove o registro do arquivo de configuração e não tem desfazer.

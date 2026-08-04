# Autenticação por Chave Privada

Autenticação por chave pública é geralmente preferível à autenticação por senha em ambientes onde você tem controle do servidor. A razão principal é simples: a chave privada nunca sai da sua máquina.

---

## Como funciona o protocolo

O método `publickey` (RFC 4252 §7) opera assim:

1. O cliente informa ao servidor qual chave pública quer usar.
2. O servidor verifica se ela está em `~/.ssh/authorized_keys` (ou onde o `AuthorizedKeysFile` aponta).
3. Se autorizada, o servidor envia um desafio.
4. O cliente assina o desafio com a chave **privada** local e manda a assinatura de volta.
5. O servidor verifica a assinatura com a chave pública. Assinatura válida = autenticado.

A chave privada nunca trafega pela rede — só a prova criptográfica de que você a possui.

---

## Algoritmos suportados

| Algoritmo | Observação |
|---|---|
| **ED25519** | Recomendado para chaves novas. Rápido, compacto e sem parâmetros que possam ser configurados de forma insegura. |
| **RSA** | Amplamente compatível. Use no mínimo 3072 bits; 4096 é preferível. |
| **ECDSA P-256** | Funciona, mas ED25519 é melhor escolha para fins equivalentes. |
| **ECDSA P-521** | Segurança maior que P-256, mesma ressalva sobre preferir ED25519. |

---

## Gerar um par de chaves

```bash
# ED25519 (recomendado)
ssh-keygen -t ed25519 -C "seu@email.com" -f ~/.ssh/id_ed25519

# RSA 4096 bits
ssh-keygen -t rsa -b 4096 -C "seu@email.com" -f ~/.ssh/id_rsa
```

O comando vai pedir uma passphrase. Usar passphrase é fortemente recomendado — se o arquivo da chave privada for comprometido, a passphrase é o que o torna inutilizável. O RusTTY suporta chaves protegidas por passphrase.

---

## Adicionar a chave ao servidor

```bash
# Via ssh-copy-id (precisa de acesso inicial por senha)
ssh-copy-id -i ~/.ssh/id_ed25519.pub usuario@servidor

# Manualmente
cat ~/.ssh/id_ed25519.pub | ssh usuario@servidor \
  "mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys"
```

As permissões importam — o OpenSSH recusa chaves de arquivos com permissões abertas demais:

```bash
chmod 700 ~/.ssh
chmod 600 ~/.ssh/authorized_keys
```

---

## Configurar no RusTTY

1. No formulário do host, selecione **"Chave Privada"** como método de autenticação.
2. Informe o caminho completo para o arquivo da chave privada (ex: `C:\Users\Vitor\.ssh\id_ed25519`).
3. Se a chave tiver passphrase, informe no campo correspondente. É tratada com as mesmas proteções de memória aplicadas a senhas.

---

## Configuração do servidor

Confirme que o `sshd_config` tem:

```
PubkeyAuthentication yes
AuthorizedKeysFile .ssh/authorized_keys
```

Em instalações modernas do OpenSSH isso já é o padrão. Para verificar o que está efetivamente ativo:

```bash
sudo sshd -T | grep -E "pubkeyauthentication|authorizedkeysfile"
```

---

## Algumas práticas que valem a pena

- **Nunca copie a chave privada para outros lugares.** A chave pública é o que você distribui; a privada fica só na sua máquina.
- **Use passphrase em todas as chaves.** Chave sem passphrase é usável por qualquer pessoa com acesso ao arquivo.
- **Restrinja chaves por IP no `authorized_keys` se possível:**

```
from="192.168.1.50",no-port-forwarding ssh-ed25519 AAAA... usuario@estacao
```

Isso limita o uso da chave ao IP especificado, mesmo que o arquivo seja comprometido.

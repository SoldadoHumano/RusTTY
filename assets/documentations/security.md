# Segurança e Proteção de Credenciais

Um gerenciador de conexões SSH precisa armazenar senhas. Isso cria uma responsabilidade clara: se o arquivo de configuração vazar, as credenciais não podem ser utilizáveis por quem as roubou. Esta seção explica como o RusTTY lida com isso.

---

## 1. Criptografia em Repouso: DPAPI + AES-256-GCM

### Windows Data Protection API (DPAPI)

O mecanismo central de proteção das credenciais é a **Windows Data Protection API (DPAPI)**, via `CryptProtectData` / `CryptUnprotectData`. A ideia por trás disso é simples: em vez de inventar um esquema de criptografia próprio com chave embutida no binário (o que seria facilmente reversível), o RusTTY delega a geração e custódia da chave ao próprio Windows.

A DPAPI deriva uma chave mestra a partir da autenticação da sessão de usuário atual. Na prática, isso significa:

- Dados protegidos por um usuário são ilegíveis em outra conta, mesmo que essa conta seja administradora.
- Copiar o arquivo de configuração para outro computador não é suficiente para ler as credenciais — a derivação da chave depende de artefatos que existem apenas na sessão do usuário original.

### AES-256-GCM

Antes de passar pela DPAPI, as credenciais já são criptografadas com **AES-256-GCM**:

- **AES-256** é a cifra padrão do NIST para dados sensíveis (FIPS 197).
- **GCM** adiciona autenticação ao ciphertext via MAC. Isso significa que se qualquer byte do arquivo for modificado — por corrupção ou por tentativa de injeção — a decriptação falha imediatamente. O arquivo adulterado simplesmente não abre.

Essas duas camadas juntas cobrem o cenário de arquivo roubado do disco: sem a sessão do usuário original, o conteúdo não é acessível.

---

## 2. Credenciais em Memória

Ter o arquivo protegido não é suficiente se a senha ficar exposta na memória RAM por mais tempo do que o necessário.

**Como o RusTTY lida com isso:**

Credenciais carregadas do disco ficam dentro de um `SecretString`, que mantém o valor criptografado em memória. Elas são expostas em texto claro apenas na janela mínima necessária para autenticar no servidor SSH. Logo depois, o buffer que continha a senha em claro é sobrescrito com zeros antes de ser liberado.

Essa zerização é feita via crate [`zeroize`](https://docs.rs/zeroize), que usa semântica de escrita volátil para garantir que o compilador não otimize a operação fora (o que aconteceria com uma atribuição normal para um valor que nunca é lido depois).

Isso protege contra processos de usuário vasculhando a memória do processo RusTTY depois que uma sessão SSH foi estabelecida. Não protege contra comprometimento de ring-0 — veja a seção **Modelo de Ameaça** para esse caso.

---

## 3. Autenticação por Chave Privada

Além de senha, o RusTTY suporta autenticação por chave pública SSH. Do ponto de vista de segurança, isso é preferível porque a chave privada nunca sai da sua máquina.

O protocolo (`publickey` auth method, RFC 4252 §7) funciona assim: o servidor envia um desafio, o cliente assina com a chave privada e manda a assinatura de volta. O servidor verifica com a chave pública. A chave privada nunca trafega.

**Formatos suportados:** ED25519, RSA e ECDSA (P-256 / P-521). ED25519 é o recomendado para chaves novas.

Para que funcione, a chave pública precisa estar em `~/.ssh/authorized_keys` no servidor de destino, e o `sshd_config` do servidor precisa ter:

```
PubkeyAuthentication yes
AuthorizedKeysFile .ssh/authorized_keys
```

Passphrases em chaves protegidas são suportadas e tratadas com as mesmas políticas de proteção em memória das senhas.

---

## 4. Sem Telemetria

O RusTTY não se comunica com nenhum servidor externo além das conexões SSH que você mesmo inicia. Não há analytics, sincronização em nuvem, ou qualquer dado sendo enviado pra fora. Tudo fica em `%APPDATA%\ByVitor\RusTTY\` no seu disco.

O código é open source e auditável se você quiser confirmar isso.

---

## 5. Verificação de Host Key (Limitação Atual)

O RusTTY ainda aceita a chave pública do servidor sem verificar contra um banco de hosts conhecidos. Isso é equivalente ao `StrictHostKeyChecking=no` do OpenSSH — funciona bem em redes controladas, mas em redes onde você não confia no roteamento, abre espaço para MITM no handshake.

A verificação de `known_hosts` está planejada para uma versão futura. Por enquanto, evite usar o RusTTY em redes públicas sem VPN para conexões que importam.

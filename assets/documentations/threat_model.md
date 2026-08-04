# Modelo de Ameaça e Limitações de Segurança

O RusTTY implementa várias camadas de proteção para credenciais em repouso e em uso. Mas toda implementação de segurança parte de um conjunto de premissas sobre o ambiente onde roda — e é importante que essas premissas estejam documentadas, especialmente porque o RusTTY é open source e você está no controle do que faz com ele.

Esta página explica o que o projeto foi desenhado para proteger, onde ele deixa de ser suficiente por si só, e o que faz sentido fazer no nível do host se você usa o RusTTY em ambientes que precisam de segurança mais rigorosa.

---

## O que o RusTTY assume sobre o ambiente

O projeto parte do princípio de que está rodando em uma máquina sob seu controle e com acesso restrito. Isso significa:

- O sistema operacional não está comprometido.
- Não há malware com privilégios elevados (rootkits, drivers maliciosos) em execução.
- Somente você — ou quem você autorizou — tem acesso à sessão Windows onde o RusTTY opera.
- O hardware não foi adulterado (keyloggers físicos, implantes, etc.).

Essas são premissas razoáveis para a grande maioria dos casos de uso. Se alguma delas não vale pro seu ambiente, vale a pena ler a seção de limitações abaixo e avaliar controles adicionais.

---

## O que o projeto protege

| Ameaça | Como o RusTTY lida |
|---|---|
| Roubo do arquivo de configuração do disco | AES-256-GCM + DPAPI: o arquivo é ilegível fora da sessão do usuário original |
| Adulteração do arquivo de configuração | O MAC do GCM invalida a decriptação se qualquer byte for modificado |
| Processo de usuário vasculhando a memória RAM | Credenciais ficam criptografadas em memória e são zeradas imediatamente após o uso |
| Interceptação de tráfego SSH na rede | Criptografia de transporte do protocolo SSH (DH + AES) |
| Outro usuário Windows acessando a configuração | A chave DPAPI é vinculada exclusivamente ao usuário proprietário |

---

## Onde o RusTTY não é suficiente por si só

Isso não é uma crítica ao projeto — é só a realidade de qualquer aplicação rodando em userspace. Nenhum software de usuário consegue se proteger de um sistema operacional comprometido.

### Comprometimento em Ring-0

Se um adversário tiver execução de código no kernel do Windows (via driver malicioso, exploit de escalação de privilégio ou rootkit), ele tem acesso irrestrito à memória de qualquer processo. Nesse cenário, zerização de credenciais e DPAPI não ajudam — a chave pode ser lida antes de ser apagada, e o token de identidade do usuário pode ser impersonado.

Se isso é uma ameaça real no seu contexto, algumas coisas ajudam no nível do host:

- Manter o Windows e drivers atualizados (CVEs de ring-0 são o caminho mais comum pra esse tipo de comprometimento).
- Habilitar **Secure Boot** + **UEFI Measured Boot** pra bloquear drivers não assinados na inicialização.
- Usar **HVCI (Hypervisor-Protected Code Integrity)** disponível no Windows 11, que impede que módulos de kernel não verificados sejam carregados.
- Auditar periodicamente o que está rodando com `driverquery /v` ou o **Autoruns** do Sysinternals.

### Acesso físico não autorizado

Com a sessão Windows desbloqueada, qualquer pessoa com acesso físico à máquina pode usar o RusTTY diretamente, copiar os arquivos de configuração ou instalar keyloggers de software. Com a máquina desligada e acesso ao disco, a DPAPI ainda protege os arquivos de configuração contra leitura offline — mas um cold boot attack pode recuperar conteúdo da RAM em janelas de minutos a horas dependendo dos módulos de memória.

Medidas básicas que ajudam:

- Bloqueio automático de tela após inatividade curta.
- **BitLocker** com TPM + PIN pra proteção do disco em repouso.
- Desabilitar boot por mídia externa na UEFI (com senha na UEFI).

### Malware rodando sob o seu usuário

Um trojan ou RAT rodando com as suas permissões de usuário tem acesso ao mesmo sistema de arquivos que o RusTTY, consegue monitorar eventos de teclado e pode aguardar uma sessão SSH aberta pra injetar comandos. Não há muito que o RusTTY possa fazer contra isso — é um problema da camada do SO.

Se você precisa de isolamento maior, vale considerar um perfil de usuário Windows separado, usado exclusivamente para gerenciar SSH, sem instalar software de uso geral nele.

---

## Em resumo

O RusTTY foi escrito pensando em segurança, mas como todo software de usuário, depende de um host saudável pra funcionar como esperado. As proteções implementadas cobrem bem os cenários mais comuns — arquivo roubado, memória vasculhada por processo não privilegiado, tráfego interceptado. Para ambientes com requisitos mais sérios, os controles de segurança no nível do host são o complemento necessário.

A fonte está disponível pra auditoria. Se você encontrar um problema ou tiver sugestões de melhoria, contribuições são bem-vindas.

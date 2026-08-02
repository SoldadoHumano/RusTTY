# Gerenciando Conexões (Hosts)

No jargão da tecnologia, um **Host** é simplesmente a máquina de destino: o computador, servidor ou roteador ao qual você quer se conectar.

O painel principal do RusTTY funciona como a sua agenda de contatos de servidores. Vamos aprender como dominá-la.

## O Formulário de Novo Host
Ao clicar em "Novo Host", você será recebido por um formulário prático:
1. **Nome / Apelido:** Como você quer chamar esse servidor? É altamente recomendável adotar um padrão, como `[PROD] Banco de Dados` ou `[CASA] Raspberry Pi`. Nomes descritivos poupam muito tempo.
2. **Endereço:** Pode ser um endereço numérico (IP como `192.168.0.15`) ou um endereço escrito (Domínio como `meuservidor.com`). 
    - *Atenção:* Por padrão, o aplicativo espera números IP para prevenir erros de digitação. Se você precisa digitar letras (um domínio), certifique-se de marcar a caixinha **"Permitir Domínios"** que fica logo abaixo!
3. **Porta:** O protocolo SSH costuma operar na porta `22`. Mude apenas se o administrador da rede informar outra porta.
4. **Usuário e Senha:** O nome da sua credencial de entrada. Pode ficar tranquilo, a sua senha será blindada assim que você clicar em "Salvar".
5. **Habilitar Teste ICMP:** O famoso "Ping". Se marcado, o RusTTY verificará o status online deste host constantemente na tela inicial.

## Quando usar o Quick Connect (Conexão Rápida)?
Você está na casa de um amigo ou prestando suporte em uma empresa e precisou conectar rapidamente no roteador deles. Você não quer salvar o IP no seu aplicativo para sempre, certo?
- Vá na aba **Conexão Rápida**.
- Digite os dados e conecte instantaneamente.
- Assim que você fechar a janela do terminal, é como se ele nunca tivesse existido. Nenhum rastro será salvo no seu disco!

## Monitoramento Automático (ICMP)
Na aba "Home", se você habilitou o ICMP para seus servidores, o RusTTY fará uma checagem em segundo plano periodicamente (geralmente a cada um minuto).
- Uma **Luz Verde** indicará que o servidor está vivo e respondendo rápido.
- Uma **Luz Vermelha** indicará que a máquina está desligada, sem internet ou bloqueando pings.
Essa é uma forma visual incrível de checar a saúde dos seus projetos de relance assim que abrir o RusTTY!

## Editando e Excluindo
Se as senhas mudarem ou os endereços forem atualizados, não precisa criar o Host de novo. Clique no botão de edição (O ícone de Lápis) ao lado do host na tela principal e o formulário aparecerá carregado com os dados.
Se precisar apagar, basta usar a Lixeira. Mas atenção: por segurança, o RusTTY fará você confirmar a ação com uma tela de alerta, pois a deleção é permanente.

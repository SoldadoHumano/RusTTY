# Como Usar o Terminal

Depois de se conectar a um Host, o RusTTY abrirá uma nova janela com a clássica "tela preta". Apesar da simplicidade visual, o motor por trás do nosso terminal foi construído para ser poderoso e extremamente veloz.

Aqui está um guia detalhado para você se tornar um mestre na operação do terminal.

## Copiando e Colando de Forma Inteligente
Terminais clássicos e o famoso atalho `Ctrl + C / Ctrl + V` geralmente não se dão bem (já que no Linux, Ctrl+C serve para cancelar comandos). No RusTTY, nós adotamos o padrão de ferramentas modernas:
- **Copiar:** Use o mouse para selecionar o texto na tela. Depois, basta pressionar `Ctrl + Shift + C`.
- **Colar:** Copiou algum comando no seu navegador? Pressione `Ctrl + Shift + V` para colá-lo instantaneamente dentro do terminal.
- Você também pode simplesmente clicar com o **Botão Direito do Mouse**. Se você tiver texto selecionado, ele será copiado. Se não houver nada selecionado, o texto que está na sua área de transferência (clipboard) será colado. Muito mais rápido!

## O Poder da Paleta de Comandos (Command Palette)
Inspirado em editores de código profissionais (como o VSCode), nós incluímos uma barra de ferramentas rápida acessada pelo teclado.
- Pressione o atalho configurado. Por padrão, é `Ctrl + .` (ponto final). Você pode alterar a tecla na aba de Configurações do gerenciador!
- Uma janela elegante se sobreporá ao terminal.
- **Copy All (Copiar Tudo):** Útil quando você rodou um diagnóstico enorme e precisa mandar todo o texto para o seu colega ou salvar em um documento. Com um clique, absolutamente toda a tela e histórico são copiados.
- **Copy Last (Copiar Últimas X Linhas):** O servidor deu erro na inicialização e soltou mil linhas? Digite o número "100" na caixinha, aperte "Copy Last" e apenas as 100 linhas mais recentes do erro serão copiadas!
- **Clear Terminal (Limpar Terminal):** Apaga tudo da tela e também remove o histórico (scrollback). Excelente para quando você vai começar uma nova tarefa e quer focar apenas no que virá a seguir.

## Histórico de Rolagem (Scrollback)
Tudo o que o servidor enviar para você ficará salvo temporariamente no "Scrollback".
- Use a rodinha do mouse (`Scroll`) para subir e descer pela tela e inspecionar linhas antigas.
- Para rolar de forma mais rápida, você pode configurar quantos "pulos" o mouse dá por vez na tela de Configurações (A aba de engrenagem). Mudar de 1 para 3 linhas costuma tornar a navegação em grandes arquivos muito mais confortável.

## Redimensionamento Fluído
O motor do RusTTY percebe imediatamente se você tentar aumentar, maximizar ou encolher a janela. Ele notifica o servidor remoto para ajustar as letras automaticamente, evitando que os textos "quebrem" ou fiquem espremidos, independentemente do monitor que você estiver usando.

## Fechando com Segurança
Terminou o que tinha que fazer? Não precisa procurar comandos de saída se não quiser. 
Ao simplesmente fechar a janela do terminal no botão "X", o RusTTY cuidará de informar ao servidor para encerrar sua sessão ativamente de forma segura, evitando processos "fantasmas" travados do outro lado.

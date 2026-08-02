# Personalização Visual

O terminal clássico é notoriamente conhecido por ser monótono (o clássico fundo preto com letras brancas ou verdes). Porém, administradores de rede e programadores leem milhares de linhas de texto todos os dias. Uma cor bem colocada pode reduzir drasticamente o tempo que você leva para achar um erro ou um endereço importante.

Por isso, o RusTTY possui o motor de personalização **Customization**, acessível pelo ícone de Pincel/Paleta no menu lateral.

## Destaque de Palavras Específicas (Keywords)
Imagine ler 5.000 linhas de um arquivo de diagnóstico tentando achar o exato momento em que algo falhou. Com as Keywords, o RusTTY faz o trabalho duro para você.

1. Navegue até a aba Personalização e adicione uma nova Keyword (Palavra-Chave).
2. Escreva, por exemplo, `ERROR` e selecione a cor Vermelha.
3. Se quiser garantir, crie também `WARNING` e coloque na cor Amarela, e `SUCCESS` em Verde.

Você também tem a opção de marcar **"Ignorar Maiúsculas/Minúsculas" (Case Insensitive)**. Se essa caixa estiver marcada, a regra de colorir vai funcionar mesmo se o servidor escrever `Error`, `error`, ou `ERROR`. O destaque aparecerá imediatamente na tela assim que o texto for impresso, em tempo real.

## Inteligência para Redes: Destaque de Endereços IP
Para especialistas que mexem muito com infraestrutura, ver IPs pulando no meio de parágrafos normais é o sonho de consumo.

O RusTTY reconhece automaticamente qualquer endereço IPv4 na tela. Mas a melhor parte é a customização por trás disso:

- **Modo Cor Única (Unified):** Todos os endereços IPs na tela receberão a mesma cor de destaque (por exemplo, Azul Claro). Simples e elegante.
- **Modo Inteligente Dividido (Split Mode):** É aqui que a mágica acontece. Se você ativar essa opção, o aplicativo saberá distinguir a classe do IP!
  - **IPs Públicos:** Endereços da internet (como o IP de sites ou servidores Cloud) serão coloridos com a "Cor Pública".
  - **IPs Privados:** Endereços de rede local e doméstica (Como redes 192.168.X.X ou 10.X.X.X) receberão a "Cor Privada".
  - *Exemplo Prático:* Se você estiver lendo logs de tráfego de um servidor, conseguirá ver batendo o olho, pelas cores, se o ataque/visita veio da internet ou de outro computador da rede local.

As alterações de customização valem apenas para os novos terminais abertos. Portanto, certifique-se de salvar suas cores e depois abrir a janela de conexão para ver o resultado na tela!

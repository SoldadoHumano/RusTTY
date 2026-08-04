# Personalização Visual do Terminal

O RusTTY tem um sistema de destaque de texto que permite colorir padrões específicos em tempo real conforme a saída do terminal chega. As regras são aplicadas localmente na camada de renderização — o conteúdo transmitido pelo servidor não é afetado.

O painel de personalização é acessado pelo ícone de paleta no menu lateral.

---

## Destaque de Palavras-Chave

Você define um padrão de texto e uma cor. Toda vez que esse padrão aparece na saída, o RusTTY colore as células correspondentes.

Para adicionar uma keyword: acesse **Personalização**, clique em adicionar, escreva o padrão e escolha a cor. A opção **Case Insensitive** faz o padrão casar independente de capitalização.

Alguns usos práticos:

| Padrão | Cor sugerida | Por quê |
|---|---|---|
| `ERROR` | Vermelho | Identificação imediata em logs de aplicação |
| `WARNING` | Amarelo | Alertas que precisam de atenção |
| `CRITICAL` | Vermelho intenso | Eventos de severidade máxima |
| `SUCCESS` | Verde | Confirmações de operação |
| `sudo` | Amarelo | Rastrear escalação de privilégio visualmente |

As regras são avaliadas em ordem de inserção. Se dois padrões se sobrepõem na mesma região de texto, o primeiro vence.

---

## Destaque de Endereços IPv4

O RusTTY reconhece automaticamente endereços IPv4 na saída do terminal. Dois modos disponíveis:

**Modo Unificado** — todos os IPs recebem a mesma cor, independente do range.

**Split Mode** — IPs são classificados pela faixa de alocação e recebem cores diferentes:

- **Privados** (RFC 1918 + loopback + link-local): `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, `127.0.0.0/8`, `169.254.0.0/16`
- **Públicos**: qualquer endereço fora dessas faixas

Isso é útil ao ler logs de acesso ou de firewall — dá pra ver de relance se o tráfego veio da internet ou de dentro da rede.

---

## Destaque de Endereços IPv6

A mesma lógica de modo unificado e split se aplica a IPv6, distinguindo link-local (`fe80::/10`), ULA (`fc00::/7`) e endereços globalmente roteáveis.

---

## Quando as alterações entram em vigor

As regras de personalização afetam apenas as sessões abertas **depois** que você salvar. Terminais já abertos não são reprocessados retroativamente — isso exigiria re-parsear o histórico inteiro do scrollback, o que teria custo de CPU proporcional ao tamanho do buffer e poderia causar inconsistências visuais.

Salve as alterações e reabra a conexão para ver o efeito.

# Boas Práticas e Desempenho

O RusTTY foi escrito para ser leve, mas algumas configurações têm impacto real no consumo de memória, CPU e banda de rede dependendo do seu ambiente. Essas são as principais delas.

---

## 1. Tamanho do Scrollback

O parâmetro `max_scrollback_lines` define quantas linhas o emulador retém na memória depois que saem da área visível.

Cada linha ocupa memória proporcional à largura do terminal e à quantidade de atributos de célula (cor, estilo). Em um terminal de ~220 colunas com estilo variado, cada linha pode ocupar entre 500 bytes e 2 KB. Com scrollback em 10.000 linhas e múltiplas sessões abertas, isso começa a somar.

| Configuração | Quando faz sentido |
|---|---|
| 1.000 – 4.000 linhas | Máquinas com memória limitada, uso geral |
| 4.000 – 10.000 linhas | Se você frequentemente precisa rolar por logs longos |
| > 10.000 linhas | Só se você sabe que vai precisar e tem RAM sobrando |

O padrão de 4.000 linhas cobre bem a maioria dos casos sem impacto perceptível.

---

## 2. Modo de Performance

O RusTTY renderiza a interface com um pipeline gráfico acelerado por GPU, com suavização vetorial e SVG dinâmico. Em máquinas com GPU dedicada isso é barato. Em ambientes sem aceleração de hardware — VMs sem passthrough de GPU, servidores headless, notebooks velhos — a renderização em software pode pesar.

O **Modo de Performance** nas Configurações desabilita suavização antialiased, renderização SVG e efeitos de transição. A interface fica funcionalmente idêntica, o custo de CPU cai bastante.

Vale ativar se você notar consumo alto de CPU ou bateria descarregando rápido.

---

## 3. ICMP em inventários grandes

O monitoramento de disponibilidade via ICMP funciona bem pra dezenas de hosts. Se você tem centenas de entradas no inventário e habilitar ICMP pra todas, o RusTTY vai ficar disparando pings continuamente — o que pode gerar tráfego que aciona alertas de IDS/IPS por parecer varredura de rede.

A recomendação simples: habilite ICMP só pra hosts que você realmente quer monitorar ativamente. Para o resto, o status aparece quando você tenta conectar de qualquer forma.

O interruptor global em Configurações desabilita todos os probes ICMP de uma vez se precisar.

---

## 4. Múltiplas sessões para o mesmo host

Por padrão, o RusTTY impede abrir mais de uma sessão simultânea para o mesmo host. Isso evita acesso concorrente não intencional — editar o mesmo arquivo em dois terminais ao mesmo tempo, por exemplo.

Se você tem uma razão legítima para múltiplas sessões (um canal de edição e outro de monitoramento, por exemplo), a restrição pode ser removida em Configurações.

---

## 5. Credenciais e exposição visual

O RusTTY mascara senhas na interface — você precisa clicar no ícone de olho pra ver. Mas vale o bom senso: se você está editando um host com colegas por perto, ou com gravação de tela ativa, confirme o estado dos campos antes de compartilhar a tela.

O Quick Connect não salva nada em disco, mas os campos ficam visíveis enquanto você preenche. Se o ambiente tem câmeras ou pessoas ao redor, use o fluxo de host cadastrado, onde as credenciais já ficam mascaradas.

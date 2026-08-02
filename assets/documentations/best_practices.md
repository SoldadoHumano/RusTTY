# Boas Práticas e Desempenho

O RusTTY foi programado em Rust para ser leve, mas ainda é possível fazer configurações equivocadas que exijam mais do seu computador do que o necessário. 

Reunimos aqui as dicas e configurações essenciais para manter o aplicativo voando e as suas demandas bem organizadas.

## 1. Ajuste do Limite de Histórico (Scrollback)
O **Scrollback** é a capacidade que a janela do terminal tem de lembrar dos textos antigos que já sumiram da sua tela. 
- Na aba de **Configurações (Engrenagem)**, o aplicativo te deixa escolher o limite. O padrão costuma ser 4000 linhas.
- **Por que não colocar um milhão de linhas?** Cada caractere salvo ocupa memória na sua placa de vídeo e memória RAM do PC. Se você está em um PC com pouca memória (4GB ou menos), limites enormes causarão lentidão severa na máquina.
- **A Recomendação:** Mantenha em 4.000 ou 10.000 linhas. É um número alto o suficiente para cobrir semanas de uso sem comprometer sua máquina, e suficiente para você voltar e ler quase qualquer registro útil.

## 2. O Salvamento do "Modo de Performance"
Os gráficos do RusTTY são gerados com qualidade altíssima utilizando suavização vetorial. É bonito, mas pesa na placa de vídeo.
- Se o RusTTY estiver consumindo muita bateria do seu notebook, ou se você roda ele em servidores ou máquinas virtuais antigas (sem placa de vídeo dedicada), ative o **Modo de Performance** nas Configurações!
- Ele desativará suavizações de ponta, rendering SVG dinâmico e outras firulas visuais, reduzindo drasticamente o consumo de CPU. A interface ficará um tiquinho mais simples, mas a performance vai disparar.

## 3. Cuidado com o Teste de Saúde Contínuo (ICMP)
Na tela principal, nós checamos se o Host está online fazendo "pings". 
- Se você tem apenas 5 a 15 servidores salvos, pode deixar o ICMP ativado para todos. É seguro e não vai sobrecarregar nada.
- No entanto, se você administra infraestruturas gigantes e possui **centenas** de servidores salvos no RusTTY, ativá-lo para TODOS fará o aplicativo ficar atirando pings na rede sem parar. Isso gastará banda da sua internet e poderá fazer alarmes soarem no provedor de rede por "tráfego suspeito".
- **A Recomendação:** Deixe o ICMP ativado apenas para os servidores mais críticos que você monitora o tempo todo (Servidores Principais e Bancos de Dados) e desative na tela de cadastro para aqueles que não são muito importantes.

## 4. Evite Exposição Visual
Lembre-se que o RusTTY esconde suas senhas na interface até mesmo quando você vai editar um host (você precisa clicar no olhinho para revelar). Nunca exiba ou edite as configurações com curiosos atrás da sua mesa! Se for usar o "Quick Connect", certifique-se de não haver gravações de tela abertas que capturem você colando dados confidenciais do seu cofre.

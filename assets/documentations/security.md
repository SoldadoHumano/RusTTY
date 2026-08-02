# Segurança e Privacidade de Dados

A segurança não é um recurso extra no RusTTY; ela foi o princípio fundamental desde a primeira linha de código. Sabemos que um gerenciador de conexões guarda as "chaves do castelo" (suas senhas de servidores), e um vazamento desses dados seria catastrófico.

Abaixo, explicamos como protegemos você, sem usar jargões excessivamente complicados.

## 1. O Cofre Intransponível do Windows (DPAPI)
A maioria dos aplicativos simples salva suas senhas em um arquivo de texto. Isso significa que se um vírus roubar aquele arquivo, o hacker terá todas as suas senhas.

Para impedir isso, o RusTTY jamais salva senhas "limpas" no disco. Nós utilizamos a **DPAPI (Data Protection API)**, que é o cofre oficial do sistema Windows (Windows Credential Manager). 
- O Windows cria uma chave mestra altamente complexa vinculada *exclusivamente* à sua conta de usuário.
- Mesmo que alguém roube o seu arquivo de configurações ou o disco rígido inteiro, é **fisicamente impossível** ler as senhas em outro computador ou em outro usuário. As chaves de decodificação só existem enquanto o seu usuário original do Windows estiver logado.

## 2. Criptografia AES-256-GCM
Não paramos apenas no cofre do Windows. Antes de enviar qualquer arquivo para o seu disco rígido, usamos a criptografia **AES-256-GCM**.
- **AES-256:** É o padrão de criptografia aprovado pelo governo dos Estados Unidos para guardar documentos ultrassecretos (Top Secret). 
- **GCM (Galois/Counter Mode):** É uma tecnologia que age como um "lacre de segurança inviolável". Se um vírus tentar alterar qualquer letra dentro do arquivo de senhas, o GCM percebe imediatamente que o arquivo foi violado e recusa a leitura, prevenindo que invasores injetem códigos maliciosos no seu aplicativo.

## 3. Blindagem de Memória RAM (Zerização)
Um perigo moderno muito comum são programas espiões que não olham o disco rígido, mas sim a Memória RAM enquanto o aplicativo está aberto.

Para lidar com isso, o RusTTY utiliza um recurso chamado **Protected Memory**.
- Quando você digita a sua senha (ou quando o aplicativo a lê do disco), ela é imediatamente criptografada na memória RAM em milissegundos.
- No momento exato em que você clica em conectar, a senha é descriptografada em uma área minúscula por uma fração de segundo, enviada para o servidor e, logo após o uso, nós passamos uma "borracha virtual" (**Zerização**) em cima dela.
- Nós escrevemos zeros em cima dos dados na memória instantaneamente, garantindo que nenhum programa espião vasculhando o seu computador consiga pescá-la no futuro.

## 4. Totalmente Offline e Privado
O RusTTY não exige que você crie contas online. Não fazemos login em nuvem, não guardamos seus dados nos nossos servidores e não enviamos estatísticas de uso secretas.
Absolutamente **tudo** sobre os seus servidores fica armazenado exclusivamente no seu próprio disco rígido local, sob o seu controle total. Você é o dono dos seus dados.

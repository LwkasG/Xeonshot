# Xeonshot: compilação e execução em Xeon de clock baixo

## Evidência atual

O Hookshot traduz o executável AArch64 para C antes da instalação. O host executa o código recompilado e implementa as interfaces do Horizon, enquanto o renderer traduz e reexecuta os comandos de GPU em Vulkan. Os três caminhos têm custos diferentes. A preparação de pipelines Vulkan na inicialização usa o compilador do driver e também precisa ser medida separadamente do tempo de build do C. [Arquitetura](../design.md#3-architecture).

No Xeon E5-2686 v4, o llvm-mingw Clang 23.1.5 retornou `-target-cpu broadwell` para `-march=native`. O build da DLL registrou `broadwell_broadwell_861cd1e4e7` nos flags que identificam os objetos. Os testes de seleção de CPU e das opções de compilação leve passaram. A comparação de desempenho ainda usa o executável instalado de referência e a DLL reconstruída localmente. [Seleção de CPU](../../nx2/host_cpu.py).

No perfil independente do percurso curto, a thread principal consumiu em média 8,37 ms de CPU e ficou bloqueada por 20,92 ms nos frames abaixo de 50 ms. Nos frames longos, os números médios foram 3,91 ms de CPU e 38,49 ms bloqueada. Nenhum dos 66 eventos de espera longa por um núcleo virtual coincidiu com os frames longos nessa janela. Esses contadores têm sobreposição e são de uma rodada com amostragem, anterior à volta completa. Na volta completa de calibração, alguns picos coincidiram com compilação Vulkan e outros tiveram bloqueios sem causa isolada. [Volta de referência](../tested/2026-11-21-xeon-market-lap-lucas-calibration01.md).

## Validação da cadeia de esperas na praça

Com a preparação normal ligada, as duas voltas em 5120×2160 registraram p99 de 71,58 e 66,09 ms. As duas voltas em 3440×1440 registraram 65,19 e 70,01 ms. O contador de pipelines ficou estável em todas as janelas. A resolução de 3440×1440 não deu uma melhora consistente nos frames longos, e os trajetos manuais variaram em duração e distância.

Em uma volta diagnóstica a 3440×1440, os dois piores frames foram de 112,38 e 101,63 ms, com quase todo o tempo registrado na thread principal bloqueada e sem espera relevante por pipeline. A captura com `HOS_CHAIN` alcançou o limite de 2.097.152 eventos depois de 6,32 s, cobrindo 203 intervalos que incluem 11 acima de 50 ms. Na reconstrução do caminho crítico, a espera por um núcleo virtual foi em média **30,24 ms nos 11 intervalos longos** e **1,91 ms nos 192 restantes**. Os maiores tempos de espera no recorte foram de `ModuleSystemWorker5`, `ModuleSystemWorker4` e `BaseProcExecutor4`. Núcleo virtual é a unidade de escalonamento que o Horizon apresenta ao jogo.

Este recorte muda a prioridade: identificar quais threads estão segurando os núcleos virtuais e quando as liberações atrasam os jobs. A etiqueta `running` da cadeia inclui estados sem bloqueio visível à captura. Ela não mede apenas ciclos executando na CPU. O buffer acabou antes do fim da volta, e os mapas disponíveis cobrem a DLL Vulkan. Ainda falta atribuir os demais picos e os custos do código do jogo a funções do host.

O primeiro candidato de runtime é um experimento opt-in com a política de tarefas de fundo, condicionado à confirmação de que elas seguram os núcleos em que os jobs do frame esperam. Usar `HOS_VCORE_WAITS` e uma captura curta em cada trecho crítico. Se os dados confirmarem o detentor, comparar a política atual com uma única variação de `HOS_BG_CAP` e verificar também carregamento, áudio e progresso dos jobs. Se a espera vier de threads da mesma prioridade, investigar a passagem de núcleo antes de testar esse limite. Manter a contagem de núcleos que o jogo vê e as regras de prioridades durante a comparação.

## Identificação das threads na praça

Com 18 núcleos físicos disponíveis, o mesmo save isolado e saída a 3440×1440, duas capturas curtas de `HOS_CHAIN` cobriram partes distintas da sequência automatizada de saltos e giros. As duas sequências terminaram sem diálogo. A espera por núcleos virtuais no caminho crítico ficou em média em 36,28 ms nos 16 intervalos de pelo menos 50 ms e em 1,96 ms nos outros 232. Esses números medem os intervalos de apresentação do host durante o diagnóstico.

`ModuleSystemWorker4`, `ModuleSystemWorker5`, `BaseProcExecutor4` e `BaseProcExecutor5` esperaram por liberações de `ParallelDecompThread` e `AudioFrameworkThread` nos dois recortes. No pior intervalo, de 111,22 ms, as esperas por núcleos virtuais somaram 87,14 ms no caminho crítico. O evento temporizado também mostrou `BaseProcExecutor5` (prioridade 16) esperando 15,8 ms por `ParallelDecompThread` (19), que liberou o núcleo 15,5 ms depois de receber o pedido. O tempo de espera que termina numa liberação pode incluir vários titulares do mesmo núcleo.

O primeiro A/B usa o limite explícito `HOS_BG_CAP=5` com os seis núcleos virtuais existentes. A política aplica o limite às threads com valor de prioridade acima de 18, incluindo a descompressão. `AudioFrameworkThread`, com prioridade 4 nessa captura, segue sua política normal. Usar A → B → B → A com diagnóstico desligado para medir o efeito sobre as travadas.

[Captura inicial](../tested/2026-11-21-xeon-vcore-lucas-path01.md) · [Captura intermediária](../tested/2026-11-21-xeon-vcore-lucas-path02.md)

### A/B com limite de tarefas de fundo

Em 3440×1440, com geração de frames ativa e o profiler desligado, cada rodada comparou 560 frames base de quatro ciclos automatizados. A telemetria confirmou saltos nos quatro ciclos e nenhuma conversa em cada rodada. Os 37.496 pipelines gráficos permaneceram estáveis nas quatro janelas.

| Rodada | `HOS_BG_CAP` | FPS base | p99 | Frames ≥50 ms | Frames ≥100 ms |
|---|---:|---:|---:|---:|---:|
| [A1](../tested/2026-11-21-xeon-bgcap-lucas-a1.md) | 0 | 32,00 | 72,44 ms | 29 | 1 |
| [B1](../tested/2026-11-21-xeon-bgcap-lucas-b1.md) | 5 | 32,67 | 68,02 ms | 26 | 0 |
| [B2](../tested/2026-11-21-xeon-bgcap-lucas-b2.md) | 5 | 32,80 | 64,01 ms | 22 | 0 |
| [A2](../tested/2026-11-21-xeon-bgcap-lucas-a2.md) | 0 | 31,31 | 79,66 ms | 35 | 1 |

Com o limite, a média dos p99 caiu 13,2% (76,05 → 66,02 ms), com melhora de 6,1% no primeiro par e de 19,6% no segundo. A taxa agregada de frames de pelo menos 50 ms caiu 22,4% (108,5 → 84,2 por minuto) e o FPS base subiu de 31,65 para 32,74. A ordem dos pares produziu a mesma direção de mudança, com magnitudes diferentes. Os quatro processos chegaram ao mercado entre 69,8 e 72,1 s de execução. O áudio reportou 999 a 1.000 blocos nos primeiros 10 s e a música do mercado continuou ativa durante cada sequência. Esses indicadores confirmam saída inicial e continuidade da música indicada pelo jogo. A validação auditiva e as transições entre áreas ficam para o teste de gameplay.

`HOS_BG_CAP=5` passa a ser candidato para a validação em voltas completas e transições entre áreas. A escolha de um padrão para o Xeonshot depende de confirmar o ganho em sessões mais longas e verificar o progresso dos carregamentos e a continuidade do áudio sob carga.

### Captura ao vivo em trecho mais pesado

Na instalação v3.0.2 já aberta, Lucas jogou com saída de 5120×2160 e FG integrado 3×. A janela de 74,85 s registrou 38,99 FPS base, p99 de 66,60 ms e 127 frames de pelo menos 50 ms. A RTX ficou em 62,1% de uso médio, com máximo de 74% amostrado, a cerca de 2,88 GHz. O processo consumiu em média o equivalente ao tempo de 5,09 processadores lógicos.

Nos frames de pelo menos 50 ms, o contador de bloqueio da thread principal passou de 16,63 para 46,04 ms em média. A espera direta pela GPU passou de 2,03 para 0,60 ms, e a espera por pipeline foi zero. Esses contadores têm janelas de coleta que podem se sobrepor. O diagnóstico aponta para dependências do lado do jogo/host, com margem de GPU. O tempo direto da thread principal esperando por um núcleo virtual permaneceu baixo (0,05 ms nos frames longos). A captura atual permite que a causa esteja num worker ou em outra espera do jogo, mas não identifica o detentor.

O WPR retornou `0xc5585011` ao habilitar o perfil do sistema. A próxima atribuição por thread pode usar uma cópia com `HOS_CHAIN` e eventos temporizados de espera, na mesma área e com os mesmos ajustes gráficos. O resultado da praça continua separado deste trecho. [Registro da captura](../tested/2026-11-21-xeon-live-heavy-lucas-run01.md)

Em uma segunda captura ao vivo, no campo aberto, o jogo manteve a imagem de 5120×2160 e apresentou em uma janela com swapchain de 1707×720. A janela medida registrou 36,55 FPS base, p99 de 72,80 ms e 149 frames de pelo menos 50 ms. O padrão voltou a aparecer: 46,11 ms de bloqueio da thread principal nos frames longos contra 16,71 ms nos demais, com RTX em 59,5% de uso médio. Um registro do mesmo trecho mostrou `BaseProcExecutor0` esperando 68,5 ms por um núcleo virtual mantido por `sead::MainThread`. As duas tinham prioridade 16, fora da classe alcançada por `HOS_BG_CAP`. O log confirma a ocorrência no trecho. A correlação com um frame específico depende de uma captura temporizada. [Captura no campo aberto](../tested/2026-11-21-xeon-live-field-lucas-run01.md)

### Validação da configuração no Campo de Hyrule

O limite de tarefas de fundo foi carregado em uma cópia isolada da instalação, com o save do campo. Na comparação de desempenho, as sessões válidas usaram o mesmo ponto inicial e quatro ciclos automatizados. A primeira referência foi descartada após movimento manual antes do início. A ordem válida foi B1 → A1 repetida → B2 → teste do spin → A2.

| Rodada | Ajuste | FPS base | p99 | Frames ≥50 ms |
|---|---|---:|---:|---:|
| [A1 repetida](../tested/2026-11-21-xeon-field-cap-lucas-a1r.md) | Padrão | 31,85 | 65,79 ms | 48 |
| [B1](../tested/2026-11-21-xeon-field-cap-lucas-b1.md) | `HOS_BG_CAP=5` | 30,82 | 69,84 ms | 59 |
| [B2](../tested/2026-11-21-xeon-field-cap-lucas-b2.md) | `HOS_BG_CAP=5` | 32,33 | 68,11 ms | 41 |
| [A2](../tested/2026-11-21-xeon-field-cap-lucas-a2.md) | Padrão | 31,97 | 66,88 ms | 48 |
| [C1](../tested/2026-11-21-xeon-field-cap-lucas-c1.md) | `HOS_VCORE_SPIN_US=50` | 31,99 | 71,68 ms | 48 |

O limite piorou o p99 nos dois pares, de 66,34 ms para 68,98 ms na média. O spin de 50 µs também piorou o p99 na rodada exploratória. O arquivo instalado de ambiente foi restaurado a partir do backup. O ganho medido na praça continua válido para aquela cena.

Na captura instrumentada de 4,41 s no campo, os 10 intervalos de pelo menos 50 ms tiveram em média 29,78 ms de espera por núcleos virtuais no caminho crítico, contra 2,14 ms nos demais 131. `ParallelDecompThread` e `AudioFrameworkThread` reapareceram nos pontos de liberação. Num intervalo de 70,28 ms, o caminho teve 51,89 ms de espera por núcleos virtuais; o log temporizado registrou 10,3 ms entre o pedido e a liberação de `ParallelDecompThread` para `BaseProcExecutor4`. O candidato específico é `HOS_NATIVE_ZSTD=1`, modo em que o descompressor nativo devolve o núcleo enquanto trabalha. A investigação do projeto registra travamentos de carregamento com esse modo num Arc 155H, então qualquer adoção exige checagem de transições no hardware alvo. [Captura instrumentada](../tested/2026-11-21-xeon-field-vcore-lucas-path01.md) · [Modos do Zstandard](../../host/hos_zstd.c)

## Ensaios que separam as causas

1. **Aquecimento e bloqueios.** Medir duas voltas no mesmo processo após a preparação normal de pipelines. Na variante diagnóstica, usar a captura de frames lentos e a cadeia de waits para localizar quem libera a thread principal. Registrar separadamente primeira passagem, cache aquecido, tempo de CPU, tempo de gravação e espera por leitura da GPU.
2. **Compilação do código do jogo.** Se a CPU realmente estiver executando código recompilado no caminho dos frames longos, gerar uma base do host com mapa de símbolos. Preservar a DLL Vulkan, o alvo Broadwell, o estado inicial de cache e as opções do jogo. Usar o perfil da volta para escolher funções quentes e testar `HOOKSHOT_CHUNK_OPT=<lista>` com `-O2` somente nos chunks selecionados. Os trechos quentes existentes já recebem `-O2` e podem usar PGO. Medir o tempo de build, o pico de memória e a mudança no custo dessas funções durante o jogo.
3. **Escalonamento.** Se os frames longos coincidirem com hand-offs, barreiras ou perda de CPU, testar uma política por vez, com os switches A/B existentes. Registrar tempo em núcleo virtual e tempo bloqueado. Preservar as regras de afinidade e prioridade do Horizon, e conferir gameplay antes de sugerir qualquer novo padrão. Uma CPU com muitos núcleos físicos continua sujeita a dependências seriais.
4. **Renderer.** Se a espera vier de criação de pipelines, upload ou comandos Vulkan, manter os experimentos nessa camada. O ganho precisa aparecer no frame completo. Para a hipótese de conversão de textura em compute, usar as capturas que representam o conteúdo no submit e validar o resultado contra o caminho da CPU.

A base do host e os símbolos ficam no PC. Rodadas de comparação passam pela fila em modo `timing`, com o mesmo save e a mesma volta completa. As medições de ganho usam o profiler desligado. [Build, perfil e A/B](../development.md) · [Plano de estabilidade](xeonshot-plano-estabilidade.md).

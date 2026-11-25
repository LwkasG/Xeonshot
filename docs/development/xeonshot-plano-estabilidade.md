# Xeonshot: estabilidade dos frames em movimento

O primeiro objetivo é reduzir os engasgos ao andar, girar a câmera e entrar em áreas novas. O segundo é usar a margem da RTX 5080 para entregar comandos mais cedo e, depois de estabilizar o jogo, sustentar melhor qualidade gráfica.

## Ponto de partida

- Na RTX 5080 com o Xeon E5-2686 v4, as duas rodadas de referência em Castle Town, com câmera parada, registraram 35,36–36,43 FPS base e p99 de 35,38–39,57 ms. Em uma coleta anterior com movimento no mercado, o log registrou janelas de 28,4–39,9 FPS base e uso da GPU de 51–69%. A coleta em movimento foi observacional e serve para escolher o percurso inicial.
- Desligar a thread de gravação reduziu o FPS base de 38,27 para 30,66 no mercado. O [preset do Xeonshot](xeonshot-market.env) mantém replay e gravação em threads dedicadas, replay por segmentos desligado e descritores padrão.
- No perfil com símbolos, `build_descriptors` ocupou 7,3% das amostras em execução da thread de replay. A variante que evitou acessos repetidos a dados locais dessa thread ficou dentro da variação entre rodadas. Falta um perfil do percurso em movimento que explique os frames longos relatados durante o jogo.

## 1. Reproduzir o stutter em jogo

Usar o save do mercado em uma cópia local. Começar com um trecho parado para referência. O benchmark principal percorre uma volta completa no perímetro da praça, saindo e voltando ao mesmo marco, com saltos e giros de câmera que mostrem os diferentes lados do mercado. Marcar quatro pontos do circuito na ordem da volta, três ações de salto e giros em pelo menos dois lados. Na primeira captura, registrar posição e direção da câmera nesses pontos para repetir o percurso. Medir a volta inteira, registrando também a duração e normalizando os picos por minuto quando as voltas tiverem durações diferentes. Separar a travessia do portão para Hyrule Field em outra coleta. Usar Kokiri Forest com uma rotação de câmera como segundo cenário antes de promover uma alteração.

- Repetir a mesma sequência de comandos pelo [control bridge](../control.md#frames-and-inputs). Confirmar no log a área, posição, direção da câmera e eventuais diálogos. Fixar a duração das entradas em frames de jogo ou usar pontos de passagem verificados, para que as duas versões percorram a mesma rota.
- Guardar dois conjuntos de coletas: primeira passagem após uma inicialização ou cache novo, e passagens repetidas com cache aquecido. Preparações antes do título ficam fora do tempo de jogo. Compilações e uploads que ocorrem durante o percurso contam como parte da experiência e ficam anotados.
- Começar com o preset gráfico atual e Frame Generation na configuração usada para jogar. Fazer uma rodada diagnóstica separada com Frame Generation desligado e os demais controles de apresentação mantidos para localizar se o engasgo já existe nos frames base.
- Coletar `RX_VK_FRAME_TRACE=1` em todas as medições. Em uma rodada separada, usar o [perfil com símbolos](../development.md#the-profiling-mode) ou `HOS_SLOW_FRAMES` para identificar a espera nos frames longos. Usar o log de sobreposição e capturar separadamente a cadência de apresentação com Frame Generation. Registrar p50, p95, p99 e p99,9 de frame time base, quantidade de frames acima de 50 e 100 ms por minuto, FPS base, uso da GPU e pico de VRAM.

**Entrega:** uma volta completa reproduzível, CSVs com a janela de cada volta marcada e uma lista dos eventos que coincidem com os piores frames.

## Triagem no mercado

Antes de calibrar a volta completa, uma sequência curta de caminhada e giros reproduziu os engasgos. Em 5120×2160, a primeira rodada registrou 44 frames de pelo menos 50 ms, seis de pelo menos 100 ms e p99 de 63,60 ms. Em cinco dos piores frames, a rubrica de uploads ficou entre 57 e 107 ms. O log identificou texturas de 5120×2160 de cerca de 42,5 MiB sendo enviadas pelo caminho da CPU. O perfil separado com símbolos também encontrou picos de espera por pipelines durante o percurso.

Na mesma sequência, as duas rodadas exploratórias em 3440×1440 registraram p99 de 42,13 e 39,33 ms e 12 e 5 frames de pelo menos 50 ms. As duas referências em 5120×2160 registraram 63,60 e 44,69 ms e 44 e 14 frames longos. O cache e as posições dentro da praça variaram. A saída de 3440×1440 fica como candidata para a volta principal.

Desligar Frame Generation em uma rodada a 5120×2160 reduziu a carga registrada na GPU, e ainda houve oito frames base de pelo menos 50 ms. O teste separado de DLAA a 3440×1440 elevou o tempo recente de GPU de aproximadamente 8 para 10 ms por frame, mas elevou também o p99 base para 50,12 ms. O DLAA não passou no recorte exploratório. Os resultados de cada rodada estão nos [registros de teste](../tested-configurations.md).

### Volta calibrada na praça

A volta manual foi capturada numa cópia do save em Castle Town. Foram 114,2 m em 25,3 s, com retorno a 1,2 m do início, mudanças de altura e câmera voltada para os diferentes lados da praça. Com saída em 5120×2160, a captura registrou 27,25 FPS base, p99 de 110,17 ms, 63 frames de pelo menos 50 ms e oito de pelo menos 100 ms. Os piores frames da volta também envolveram esperas de pipeline e bloqueios da thread principal. A calibração foi feita em modo compartilhado e serve para fixar o trajeto, com os pontos e a direção da câmera guardados localmente. O A/B formal usa as duas resoluções na volta calibrada, com o mesmo estado inicial de cache. [Registro da volta](../tested/2026-11-21-xeon-market-lap-lucas-calibration01.md).

### Comparação com preparação normal

Com os pipelines preparados, o p99 das duas voltas em 5120×2160 ficou em 71,58 e 66,09 ms e, em 3440×1440, em 65,19 e 70,01 ms. As voltas variaram em duração e distância e a troca de resolução não mostrou melhora consistente. A coleta da cadeia de esperas indicou contenção por núcleos virtuais nos frames longos do trecho registrado. [Análise da compilação e do escalonamento no Xeon](xeonshot-compilacao-clock-baixo.md).

## 2. Atacar a causa dominante

| Se os frames longos coincidirem com… | Primeira alteração a testar | Evidência de que funcionou |
|---|---|---|
| Criação de pipeline, textura nova, upload ou pressão de VRAM | Aquecer seleções críticas antes da entrada e distribuir uploads que possam esperar, usando o orçamento de uploads e os mecanismos existentes. Manter recursos que alimentam o frame atual dentro das dependências Vulkan. | Menos picos de compilação ou upload na mesma aproximação. Sem textura atrasada, piscada, crescimento contínuo de VRAM ou pausa maior ao cruzar a área. |
| GPU ociosa entre gravações, com custo elevado de comandos ou espera pela thread de gravação | Medir onde a fila de comandos esvazia e qual limite de gravação faz o replay esperar. Formular uma única alteração de batching, gravação redundante ou reutilização de descritores a partir da função quente do percurso. Testar `RX_VK_RECORDER_OWN` somente se resets de pools responderem por uma parte relevante da espera. | Menos ociosidade entre gravações, queda no tempo da etapa medida e melhora do p99 base em ambas as ordens do A/B. |
| `memsync_ms`, `late_wait_ms` ou `readback_wait_ms` elevados | Rastrear quais páginas e passes produzem a espera. Avaliar se agrupamento das cópias ou compute no caminho existente dos mirrors remove trabalho da CPU ou um retorno à memória do host. Preservar a ordem dos eventos, as capturas feitas no submit e a disponibilidade dos dados para cada leitura. | A espera específica diminui, os frames longos ficam mais raros e exposição, sombras e culling continuam corretos. |
| Frames base estáveis com picos concentrados nas apresentações com Frame Generation | Comparar a cadência de apresentação com Frame Generation desligado e depois com um multiplicador menor, mantendo a configuração de refresh, V-Sync e Reflex medida. Alterar um controle de apresentação por vez. | Menos intervalos irregulares no vídeo apresentado, sem piora do p99 base nem aumento perceptível de atraso de resposta. |

Usar a primeira linha que os dados do percurso sustentarem. O [pipeline do frame](frame-pipeline.md) já documenta casos em que antecipar a gravação só deslocou a ociosidade e em que responder antes a uma espera fez as sombras piscarem. Qualquer alteração precisa preservar essas dependências.

## 3. Usar a margem da GPU para qualidade

Criar uma variante opcional de qualidade sobre o preset atual. Aumentar um ajuste que mude sobretudo o trabalho da GPU, começando pela escala de renderização ou modo de upscaling, com distância de objetos, vegetação e sombras fixas. Se o uso ou tempo de GPU subir com frame time base estável, essa faixa pode oferecer mais qualidade. Se também subir o p99 ou aparecerem novos picos, usar a configuração anterior. Testar objetos, vegetação e distância de sombras separadamente, porque também podem aumentar os draws que o Xeon precisa preparar.

**Entrega:** um preset estável e, se a medição permitir, uma variante de qualidade. Uma utilização maior da GPU serve como evidência de que ela está fazendo mais trabalho útil. O p99 e a frequência das travadas decidem se o resultado joga melhor.

## Critério para aceitar uma melhoria

- Fazer quatro rodadas com a mesma base e uma alteração por vez, na ordem A → B → B → A. Medir pelo [job queue](../development.md#sharing-a-pc-the-job-queue) em modo `timing`, com o profiler desligado. Manter save, rota, preset, estado inicial do cache e duração da coleta comparáveis.
- Meta inicial de engenharia: **reduzir pelo menos 10% do p99 base ou 30% dos frames acima de 50 ms** no percurso que reproduz a travada. Validar a direção do ganho nos dois pares, sem queda superior a 2% no FPS base médio e sem piora consistente dos frames acima de 100 ms. Calibrar esses limites depois de medir a variação natural do percurso. Resultados abaixo da variação de referência ficam como inconclusivos.
- Conferir mercado, portão e Kokiri quanto a cintilação, paredes e objetos ausentes, sombras, exposição, HUD e estabilidade de memória. Rodar os testes sem jogo, registrar cada medição em [tested configurations](../tested-configurations.md) e preservar o comportamento dos outros perfis e plataformas até haver validação neles.

**Primeira tarefa:** calibrar a volta completa com saltos e giros, obter a referência A da volta em movimento e cruzar os piores frames com a gravação, os uploads, as leituras de GPU e a apresentação.

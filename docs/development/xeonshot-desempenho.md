# Xeonshot: desempenho no Xeon E5-2686 v4 e RTX 5080

## Método

As rodadas medidas abriram um processo novo, usaram uma cópia do mesmo save de Castle Town Market e fecharam o jogo ao terminar. A sequência aplicou os mesmos quatro ciclos de saltos laterais e giros rápidos pelo analógico direito. Cada comparação usa 560 frames base da sequência, com checagem de altura, câmera, mapa e diálogo. A triagem que alternou configurações na mesma sessão ficou fora dos resultados por causa da variação de posição e câmera.

Os 18 núcleos físicos continuaram disponíveis, com Hyper-Threading e o par 34–35 reservado para o replay. As rodadas usaram `HOS_BG_CAP=5`, o preset do mercado, a preparação normal de pipelines e a mesma base e DLL Vulkan. As medições passaram pela fila em modo `timing`.

- A comparação de Frame Generation mantém a saída e a renderização nativa em 3440×1440 e muda só o multiplicador integrado, entre 8×, 3× e desligado. A tela opera a 165 Hz.
- A comparação de qualidade sobe a imagem gerada pelo jogo de 3440×1440 para 5120×2160, mantendo geometria, sombras e Frame Generation em 8×. O swapchain continua em 3440×1440, então a imagem maior é reduzida para apresentação.
- A variante de DLSS usa Qualidade (67%, 2294×960) com saída 3440×1440 e Frame Generation integrado em 3×. A rodada extra do gravador mantém essa variante e ativa `RX_VK_RECORDER_OWN=1`.

A carga da RTX vem da NVML. A CPU é o tempo consumido pelo processo dividido pelo tempo de parede, em equivalentes de um núcleo. A taxa de frames gerados conta chamadas de apresentação do interpolador integrado. Nos cenários com interpolação, a latência medida vai da entrega do frame base à chamada de apresentação. FPS base, chamadas de apresentação e latência de entrada até a tela são medidas distintas.

## Frame Generation com reinício entre as rodadas

| Frame Generation | FPS base | p99 médio | Frames ≥50 ms/min | Chamadas de frames gerados/s | Carga da GPU | Potência da GPU |
|---|---:|---:|---:|---:|---:|---:|
| 8× | 31,40 | 70,78 ms | 89,2 | 122,8 | 59,8% | 126 W |
| 3× | 32,56 | 67,38 ms | 87,2 | 64,9 | 50,2% | 114 W |
| Desligado | 33,02 | 68,84 ms | 79,6 | 0 | 38,0% | 97 W |

Foram dois processos independentes por configuração. Os seis processos terminaram, usaram a mesma origem de save e tiveram a configuração idêntica nas demais opções. Cada sequência registrou saltos nos quatro ciclos. A câmera acumulou entre 845° e 960° conforme a rodada. O p99 variou entre as repetições, e houve um pico de pelo menos 100 ms em 3×, um com FG desligado e zero em 8×.

Desligar o FG deu o maior FPS base e a menor potência medida nas duas comparações com 8×. Usar 3× preservou interpolação com menor custo de GPU. A média das medianas entre entregar o frame base e chamar sua apresentação ficou em 25,23 ms com FG 3× e 31,31 ms com FG 8×. O valor de 8× é o solicitado e o jogo limita a geração à taxa da tela.

O [preset opcional de FPS base](xeonshot-market-performance.env) usa 3440×1440 com FG desligado. Na cópia local de Xeonshot, mesclar as linhas no `hookshot_env.txt` junto de `image.bin` e manter o upscaler do jogo a 100% no menu. Para comparar a variante intermediária, retirar a linha `RX_VK_FRAMEGEN=0` e selecionar Frame Generation integrado em 3× no menu.

[FG 3× rodada 1](../tested/2026-11-21-xeon-throughput-lucas-f3-1.md) · [FG 3× rodada 2](../tested/2026-11-21-xeon-throughput-lucas-f3-2.md) · [FG desligado rodada 1](../tested/2026-11-21-xeon-throughput-lucas-f0-1.md) · [FG desligado rodada 2](../tested/2026-11-21-xeon-throughput-lucas-f0-2.md)

## Aumentar a resolução e trocar o renderizador

| Ajuste | FPS base | p99 médio | Frames ≥50 ms/min | GPU | Potência da GPU | CPU em núcleos equivalentes |
|---|---:|---:|---:|---:|---:|---:|
| 3440×1440 nativo, FG 8× | 31,40 | 70,78 ms | 89,2 | 59,8% | 126 W | 5,94 |
| 5120×2160 nativo, FG 8× | 31,81 | 68,03 ms | 85,2 | 64,3% | 152 W | 5,99 |
| 3440×1440, DLSS Qualidade, FG 3× | 31,80 | 73,83 ms | 83,5 | 50,1% | 116 W | 5,95 |

A resolução maior usou 26 W adicionais e manteve o tempo de CPU no mesmo patamar. O p99 caiu no primeiro par (76,95 → 67,75 ms) e subiu no segundo (64,61 → 68,31 ms). É possível usar essa margem para qualidade, com ganho de FPS base pequeno no mercado e estabilidade variável. O DLSS Qualidade reduziu a carga da RTX. O efeito sobre o FPS base e o p99 variou entre as rodadas.

`RX_VK_RECORDER_OWN=1` foi confirmado no log em uma rodada com DLSS Qualidade e FG 3×. Registrou 32,20 FPS base, p99 de 66,57 ms e 82,8 frames de pelo menos 50 ms por minuto, frente a 31,80 FPS, 73,83 ms e 83,5/min na média das duas rodadas com o gravador padrão. O resultado do gravador fica exploratório.

[Nativo A1](../tested/2026-11-21-xeon-throughput-lucas-a1.md) · [Nativo A2](../tested/2026-11-21-xeon-throughput-lucas-a2.md) · [5120 Q1](../tested/2026-11-21-xeon-throughput-lucas-q1.md) · [5120 Q2](../tested/2026-11-21-xeon-throughput-lucas-q2.md) · [DLSS B1](../tested/2026-11-21-xeon-throughput-lucas-b1.md) · [DLSS B2](../tested/2026-11-21-xeon-throughput-lucas-b2.md) · [Gravador C1](../tested/2026-11-21-xeon-throughput-lucas-c1.md)

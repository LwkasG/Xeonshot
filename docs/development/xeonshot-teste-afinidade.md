# Xeonshot: teste de afinidade por núcleo físico

A comparação verifica se limitar o conjunto de núcleos físicos disponível ao processo muda a frequência de travadas em Castle Town. O Windows detecta 18 pares SMT consecutivos neste E5-2686 v4. O último par, 34–35, fica reservado para o replay em todas as rodadas.

| Ordem | Núcleos físicos | Processadores lógicos permitidos | Máscara Windows |
|---|---:|---|---|
| A | 18 | 0–35 | `0xfffffffff` |
| B | 8 | 0–13 e 34–35 | `0xc00003fff` |
| C | 6 | 0–9 e 34–35 | `0xc000003ff` |

A afinidade é aplicada antes de iniciar o jogo. O teste lê a máscara efetiva do processo do jogo e só segue se ela corresponder ao cenário. Em todas as rodadas, o Hyper-Threading continua ativo e o jogo mantém a política de núcleos virtuais. A máscara fixa reserva o mesmo par para o replay. Um ensaio com apenas um irmão SMT por núcleo fica separado.

## Protocolo

1. Usar a cópia de desenvolvimento da mesma base, a DLL Vulkan de referência, o mesmo save do mercado e a saída de 3440×1440. Manter o preset do Xeonshot, Frame Generation, preparação normal dos pipelines e o CSV de frames base. Executar pela fila em modo `timing`.
2. Após carregar o save e estabilizar no ponto inicial, executar por controle automatizado a mesma sequência de saltos e giros rápidos de câmera em uma área curta da praça. Girar pelo analógico direito. Conferir a altura de Link e o ângulo da câmera nas amostras e descartar qualquer rodada com conversa, carregamento ou saída do mapa.
3. Medir somente os frames entre o início e o fim da sequência. Comparar mediana e p99 do tempo dos frames base, FPS base e contagem de frames de pelo menos 50 e 100 ms normalizada por minuto. Anotar a contagem de pipelines no trecho para identificar aquecimento desigual. Uma captura de esperas diagnóstica roda à parte.
4. Se 8 ou 6 parecer melhor, confirmar com A → candidato → candidato → A, repetindo a sequência com estado inicial de cache equivalente. Usar a meta do [plano de estabilidade](xeonshot-plano-estabilidade.md#critério-para-aceitar-uma-melhoria). A primeira rodada de cada cenário serve para escolher o que repetir.

## Resultado no E5-2686 v4

As tentativas em que apareceu diálogo ficaram fora da comparação. Após reposicionar Link sem apertar A, o controle alternou saltos laterais com ZL+A e giros rápidos pelo analógico direito. A tabela usa os primeiros 560 frames base da sequência em cada configuração, cobrindo quatro ciclos. A telemetria registrou saltos e giros nos quatro ciclos e nenhum evento nas amostras comparadas. Os pontos de partida ficaram a até 1,1 m entre si e a câmera percorreu de 874° a 961° em ângulo acumulado. O log registrou 37.496 pipelines gráficos ao longo das três janelas.

| Núcleos físicos | Duração | FPS base | Mediana | p99 | Frames ≥50 ms (por min) | Frames ≥100 ms |
|---|---:|---:|---:|---:|---:|---:|
| [18](../tested/2026-11-21-xeon-affinity-lucas-18c01.md) | 16,94 s | 33,06 | 27,62 ms | 74,31 ms | 28 (99,2) | 1 |
| [8](../tested/2026-11-21-xeon-affinity-lucas-08c01.md) | 17,44 s | 32,10 | 29,39 ms | 65,91 ms | 26 (89,4) | 0 |
| [6](../tested/2026-11-21-xeon-affinity-lucas-06c01.md) | 19,30 s | 29,01 | 31,91 ms | 86,50 ms | 41 (127,5) | 1 |

O limite de 6 núcleos teve o menor FPS base e o maior p99. Com 8 núcleos, o p99 caiu 11,3%, a taxa de frames de pelo menos 50 ms caiu 9,9% e o FPS base caiu 2,9%. A queda de FPS ultrapassa o limite de 2% definido para aceitar um ganho. Como houve uma rodada comparável por configuração, o resultado é exploratório. A recomendação para esta máquina continua sendo usar os 18 núcleos disponíveis.

# O2Rusty

O2Rusty is a 7K VSRG inspired by osu!mania, o2jam, IIDX and LR2.

Still early in development. Open source, vibe coded, works on Windows.

---

## What it has so far

- 7-key gameplay with scroll speed, hit timing, long notes, grades
- Song select with scrolling list, tutorial included
- Support for .osz beatmap import (mania mode 3)
- Settings for controls, gameplay, visuals and audio
- Customizable note colors, note styles (rectangle, circle, arrow), column width, note height
- Column spacing, column background and column line toggles
- Audio offset calibration
- Pause menu with continue, reset and quit
- Mouse support in all menus

## Controls

Default keys: S D F Space J K L

You can rebind any key in Settings > Controls.

## How to add songs

Place .osz files or folders with .osu charts inside the `songs/` folder.

## Build

```
cargo run --release
```

Depends on Rust and Cargo. No extra setup needed.

---

# O2Rusty

O2Rusty é um VSRG de 7 teclas inspirado em osu!mania, o2jam, IIDX e LR2.

Ainda no começo do desenvolvimento. Código aberto, vibe coded, funciona no Windows.

## O que tem até agora

- Jogabilidade 7 teclas com velocidade de scroll, timing, long notes, notas
- Seleção de música com lista rolável, tutorial incluso
- Suporte a importação de .osz (mania mode 3)
- Configurações de controles, jogabilidade, visuais e áudio
- Cores e estilos de nota customizáveis (retângulo, círculo, seta), largura da coluna, altura da nota
- Espaçamento entre colunas, fundo e linha da coluna com toggle
- Calibração de offset de áudio
- Menu de pause com continuar, reset e sair
- Suporte a mouse em todos os menus

## Controles

Teclas padrão: S D F Space J K L

Dá para mudar qualquer tecla em Configurações > Controles.

## Como adicionar músicas

Coloque arquivos .osz ou pastas com charts .osu dentro da pasta `songs/`.

## Build

```
cargo run --release
```

Precisa de Rust e Cargo. Sem configuração extra.

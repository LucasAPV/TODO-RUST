# todo

Varre arquivos ou projetos inteiros atrás de tags `[TODO]` e mostra tudo em uma
TUI (tela alternativa, como `less`/`vim`). Saia com **q**.

```bash
cargo install --path .     # instala o binário `todo`

todo -f src/main.c         # um arquivo (pode repetir -f)
todo -d meu_projeto        # uma pasta, recursivamente (respeita .gitignore)
todo                       # sem argumentos: pasta atual
```

## Formato da tag

Funciona em qualquer linguagem, dentro ou fora de comentários:

```c
// [TODO]
// title:"Corrigir vazamento de memória",
// desc="parse_input() não libera o buffer em caso de erro.",
// relevance=importante
```

- `:` e `=` são aceitos; aspas são opcionais em valores simples.
- Strings podem ocupar várias linhas e aceitam escapes (`\"`, `\n`).
- Também funciona em `/* ... */`, `#`, `--`, `<!-- -->`, etc.
- Forma curta: `// [TODO] ajustar isso` vira um TODO só com título.
- Relevâncias reconhecidas (cores/ordem na lista): **importante** (urgente, alta),
  **fazendo** (doing, wip, em andamento), **pendente**, **baixa**, **feito** (done).
  Qualquer outro texto aparece como está, em roxo.

## Atalhos

| Tecla | Ação |
|---|---|
| `j` `k` / `↓` `↑` | próximo / anterior |
| `g` `G` | primeiro / último |
| `Espaço` `b` / `PgDn` `PgUp` | página |
| `Ctrl-d` `Ctrl-u` | meia página |
| `J` `K` | rolar o painel de detalhes |
| `Tab` / `Shift-Tab`, `0`–`6` | filtrar por relevância |
| `/` | buscar (título, descrição, arquivo) |
| `Esc` | limpar filtro e busca |
| `?` | ajuda |
| `q` | sair |

## Estrutura

```
src/main.rs     CLI (clap) e loop de eventos
src/scanner.rs  percorre arquivos/pastas (crate `ignore`)
src/parser.rs   extrai e interpreta os blocos [TODO]
src/app.rs      estado, filtros, busca e teclado
src/ui.rs       renderização (ratatui)
src/theme.rs    paleta de cores
```

# figma-mcp

A minimal [MCP](https://modelcontextprotocol.io) server for reading [Figma](https://www.figma.com) files
as *text*, written in Rust as a literate [org-babel](https://orgmode.org/worg/org-contrib/babel/)
program. It exposes the structure of a file, the text layers of its screens and its comments as tools an
LLM assistant can call directly, over the standard MCP `stdio` transport.

This is an unofficial project. It is not affiliated with, endorsed by or sponsored by Figma, Inc.

No third-party MCP SDK crate is used: the `JSON-RPC 2.0` wire protocol is implemented directly on top of
`tokio` and `serde_json`, following the pattern of
[org-tmetric-mcp](https://github.com/EugineKosenko/org-tmetric-mcp).

## Why

The official Figma MCP server counts every read call, and on a free (Starter) plan the budget is used up
within a single session of analysing wireframes. Reading data through the REST API with a personal access
token is subject to the same limits: they are counted per user and per plan, not per client. Measured on
a Starter plan with a Viewer/Collab seat in the plan that owns the file, `GET /files/{key}/nodes`
returned `429` after six calls, with a `Retry-After` of about 4.6 days.

This server does not get around the limits. It only spends the budget sparingly:

- the nodes of a whole list of ids are read in **one** call, `GET /files/{key}?ids=...`, instead of one
  call per node;
- every successful response is cached on disk, so repeating a request costs nothing;
- an exhausted limit (`429`) comes back as a clear message with the time to wait and the plan and seat
  type from the response headers, not as a crash.

The data is returned as text (node structure, text layers, comments), because that is what is needed to
read a wireframe: filters, columns, buttons and their order. There are no screenshots.

## Tools

- **outline** — the structure of the nodes: `id`, type and name of each node of a subtree (`ids`,
  optionally `depth`).
- **texts** — the text layers of the nodes, grouped by screen rows (top to bottom, left to right):
  filters, column headers, buttons (`ids`).
- **comments** — the comments of the file as threads: number, date, state, node with the pin offset
  (`@979:3 x=500 y=5471`) and text, replies below each one (optionally `node_id`). The offset is the
  position of the comment inside the node, so a client that knows the geometry of the nodes can find the
  screen a comment belongs to. The author name is not printed.

All tools accept an optional `file` and `refresh`. `file` is the file key or the full file URL copied
from the browser (the key is taken from the segment after `/design/`, `/file/` or `/board/`); without it
`FIGMA_FILE` is used, and if that is not set either, the tool answers with a clear error. `refresh` skips
the cache and asks Figma again. Node ids may be given in the dash form of a browser URL (`979-3`); they
are converted to the API form (`979:3`).

## Source layout

Every `.rs` file here is generated (tangled) from an `.org` file of the same purpose — edit the `.org`
source and re-tangle, never the `.rs` files directly. The prose of the `.org` files is in Ukrainian.

| org file | tangles to | purpose |
|---|---|---|
| `config.org` | `Cargo.toml` | dependencies |
| `env.org` | `.env.example` | environment variables |
| `gitignore.org` | `.gitignore` | ignored files |
| `main.org` | `src/main.rs` | the JSON-RPC/stdio protocol loop |
| `main-client.org` | `src/client.rs` | shared Figma client, cache, `429` handling |
| `main-tools.org` | `src/tools/mod.rs` | tool registry and dispatch |
| `main-tool-*.org` | `src/tools/*.rs` | one file per tool |

## Building

```sh
cargo build --release
```

## Configuration

Create a personal access token in Figma (Settings → Security → Personal access tokens); the read scopes
`file_content:read` and `file_comments:read` are enough. Copy `.env.example` to `.env` for local `cargo
run` testing only:

```
FIGMA_TOKEN=
FIGMA_FILE=
#FIGMA_CACHE=
```

`FIGMA_FILE` is optional: a default file (key or URL) for calls that do not name one. One server serves
any number of files, so the file can also be given in every call. `FIGMA_CACHE` overrides the cache
directory, which is `figma-mcp` in the system temporary directory by default.

## Registering with Claude Code

```sh
claude mcp add --scope user figma \
  -e FIGMA_TOKEN=<your personal access token> \
  -- /path/to/figma-mcp/target/release/figma-mcp
```

Add `-e FIGMA_FILE=<default file key>` if you mostly work with one file.

## License

[MIT](LICENSE)

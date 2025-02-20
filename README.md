# Hyle Oof

Small demo app for the hyle AMM

### Running

#### Frontend

- **React (Vite):**

```bash
cd web-react/
pnpm install
pnpm run dev
```
Rebuilds the app whenever a change is detected and runs a local server to host it.

#### Backend

```sh
cargo run
```

Note: You need to have a running [hyle](https://github.com/Hyle-org/hyle) node with indexer:
```sh
# in hyle repo
git checkout v0.11.1
cargo run -- --pg
```

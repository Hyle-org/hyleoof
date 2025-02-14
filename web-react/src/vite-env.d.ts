/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_SERVER_URL: string;
  readonly VITE_NODE_URL: string;
  readonly VITE_SNAP_ORIGIN: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

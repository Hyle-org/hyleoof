import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";
import App from "./App.tsx";
import { MetaMaskProvider } from "./hooks/MetamaskContext.tsx";

import { dark, light } from './config/theme';
import { ThemeProvider } from "styled-components";
createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ThemeProvider theme={dark}>
      <MetaMaskProvider>
        <App />
      </MetaMaskProvider>
    </ThemeProvider>
  </StrictMode>,
);
